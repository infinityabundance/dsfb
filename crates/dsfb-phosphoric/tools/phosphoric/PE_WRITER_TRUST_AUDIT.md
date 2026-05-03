# PE Writer Trust Audit

This file audits the PE/COFF EFI image writer at line-by-line precision, mirroring [ember/docs/EMBER_TRUST_AUDIT.md](../../ember/docs/EMBER_TRUST_AUDIT.md). The PE writer is host-profile (not trusted-profile), but it is still TCB: it produces the bootable artifact the QEMU smoke test executes.

The audit gate `check_writer_audit_lines.phos` (host program) asserts every executable line in the PE writer's source has a matching audit row here. Uncovered lines fail CI.

## Migration status (as of 2026-04-27)

Today the PE writer lives at [tools/phosphoric/write_boot_efi_from_ir.sh](write_boot_efi_from_ir.sh) (shell). After elevation item E6, the writer is rewritten as Phosphoric host-profile source under `tools/phosphoric-host/pe_writer/`. The shell version remains until parity is proven, then is moved to `archive/`.

This audit is structured against the *target* Phosphoric writer. Until E6 lands, the shell version is audited inline below; after E6, this document is rewritten line-by-line against the Phosphoric source.

## Scope

The PE writer emits:

- DOS header (64 bytes; magic `MZ`, e_lfanew pointer)
- PE signature (4 bytes; `PE\0\0`)
- COFF header (20 bytes; machine = AMD64, NumberOfSections, characteristics)
- Optional header (PE32+; magic 0x20B; ImageBase, AddressOfEntryPoint, sizes, subsystem = EFI_APPLICATION)
- Section table (40 bytes per section): `.text`, `.data`, `.reloc` only
- `.text` section content: the emitted boot_asm_v1
- `.data` section content: render command list + boot constants
- `.reloc` section content: bounded base relocations for the EFI loader

Out of scope (refused by the writer; producing such an image is a hard error):

- import table
- export table
- thread-local storage
- security directory
- exception directory
- debug directory
- delay-load directory
- .rsrc, .rdata, .pdata, .xdata, .idata, .didat sections

## Per-Function Audit (shell version, transitional)

| Function | Why it exists | Why it must be trusted today | Can it move upward? |
| --- | --- | --- | --- |
| `write_dos_header` | Emit the 64-byte DOS stub. | The stub is the first bytes UEFI reads; bad bytes brick the image. | Yes — moves into pcc.codegen.host once E5 lands. |
| `write_pe_signature` | Emit `PE\0\0`. | Required by PE spec. | Yes. |
| `write_coff_header` | Emit machine, sections count, timestamp (= 0 for repro). | Header field correctness gates loader acceptance. | Yes. |
| `write_optional_header_pe32plus` | Emit ImageBase, AddressOfEntryPoint, alignments, subsystem. | Subsystem must be EFI_APPLICATION (10). | Yes. |
| `compute_section_layout` | Compute virtual + raw addresses for `.text`, `.data`, `.reloc`. | Bad layout = image won't load. | Yes. |
| `emit_text_section` | Write `.text` from boot_asm_v1 hex stream. | The bytes here are what executes. | Yes. |
| `emit_data_section` | Write `.data` constants. | Render command list lives here; corrupt constants = corrupt frame. | Yes. |
| `emit_reloc_section` | Bounded base relocations. | UEFI loader applies these; wrong relocations = jumping into garbage. | Yes. |
| `align_to` | Round offsets up to file/section alignment. | Bad alignment = reject by loader. | No (utility) but verifiable. |
| `compute_checksum` | PE checksum (currently 0 for non-loaded modules; UEFI doesn't strictly require nonzero). | Required for some UEFI implementations. | Yes. |

## Per-Line Audit (Phosphoric version, target)

After E6, this section becomes a row per executable line of `tools/phosphoric-host/pe_writer/`. Each row records:

- file path and line number
- the operation in one sentence
- the input invariants (e.g. "section_count <= 8")
- the output invariants (e.g. "DOS header is exactly 64 bytes")
- aliasing: "this line writes to `out[ofs..ofs+N]` and to nothing else"

Until E6, this section is empty. The shell-version table above is the active audit.

## Verification

`check_writer_audit_lines.phos`:

1. Read PE writer source files (shell or Phosphoric, depending on migration phase).
2. Count executable lines.
3. Read this audit doc; count rows in the per-line / per-function table.
4. Assert one-to-one correspondence (or "covered by line range" for utility helpers).
5. Exit non-zero on any uncovered line.

## Negative-Test Coverage

The PE writer is paired with `check_direct_pe_negative_tests.sh` (today shell; after E6, `phosphoric_pe_negative_tests.phos`). Negative tests assert the writer rejects:

- malformed input IR (bad magic, bad version)
- section count > 3
- entrypoint RVA outside `.text`
- relocation outside `.reloc` section bounds
- IR hash mismatch against the `linked-artifact.txt` manifest

Each negative test is a `.phos` (post-E6) or shell input (today) plus an expected stable diagnostic code.

## Non-Goals

- Not a general PE/COFF writer. Only the EFI subset Phosphoric emits.
- No support for `.rsrc`, `.idata`, `.edata`, `.pdata`.
- No code-signing inside the writer; signing is a separate downstream tool.
- No support for non-AMD64 machines; the architecture matrix in elevation E15 / aarch64 future work would extend this.
