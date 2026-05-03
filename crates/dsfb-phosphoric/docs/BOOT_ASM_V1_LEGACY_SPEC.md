# `boot_asm_v1` — Legacy Shell Emitter Specification

This document captures the byte-level encoding produced by the legacy boot-path emitter at `tools/phosphoric/emit_boot_demo_from_phos.sh`. The emitter is the v0 backend of record while the project's Phosphoric-authored codegen reaches byte parity. Capturing the spec here serves two purposes:

1. It is the reference a reviewer reads to understand what the boot path produces today.
2. It is the contract the Phosphoric-authored codegen aligns to in order to retire the shell emitter (per recommendation R4 in the panel review).

When the Phosphoric-authored backend reaches byte equivalence on every conformance case for 30 consecutive days, the shell emitter moves to internal storage and this document becomes the formal spec the runtime consumes.

---

## 1. Output Shape

The emitter consumes a typed Phosphoric source file and produces an x86_64 assembly text file in a deliberately narrow subset of GNU AS syntax. The output is consumed by the project's PE/COFF writer at [tools/phosphoric/write_boot_efi_from_ir.sh](../tools/phosphoric/write_boot_efi_from_ir.sh) — there is no external assembler invocation.

Each emitted file has three sections in fixed order:

```
.text
   <function bodies, in declaration order>
.data
   <global rodata strings + tables>
.section .reloc
   <none — the boot image is position-fixed>
```

The text section contains all generated functions plus a small fixed prologue and epilogue.

---

## 2. Calling Convention

The emitter targets `x86_64-pc-windows-msvc`-style calling convention because the boot image runs under UEFI:

| Register | Purpose |
|---|---|
| `rcx` | First argument |
| `rdx` | Second argument |
| `r8` | Third argument |
| `r9` | Fourth argument |
| `rax` | Return value |
| Stack | Arguments 5+ |

Caller reserves 32 bytes of shadow space before each call. Callees may use the shadow space as scratch.

Floating-point registers are not used. The boot path is integer-only.

---

## 3. Function Prologue and Epilogue

Every emitted function has the same prologue and epilogue:

```
.p2align 4
<function_name>:
    pushq %rbp
    movq %rsp, %rbp
    [optional: subq $32, %rsp        # only when this function calls another]
    <body>
    [optional: addq $32, %rsp]
    leave
    retq
```

- `.p2align 4` aligns the function entry to a 16-byte boundary (mandated by the Microsoft x64 ABI for performance and by the project's PE/COFF writer for layout determinism).
- `pushq %rbp` / `movq %rsp, %rbp` establishes the frame pointer.
- `leave` / `retq` is the symmetric epilogue.

Functions that call other functions reserve 32 bytes of shadow space. Leaf functions (no calls) do not reserve shadow space. The shadow-space reservation is part of the deterministic byte layout.

---

## 4. Reserved Symbols

The emitter generates exactly four exported symbols:

| Symbol | Role |
|---|---|
| `efi_main` | UEFI entrypoint. Receives `rcx = image_handle`, `rdx = system_table_ptr`. |
| `phosphoric_demo_init` | Demo state-machine initializer. |
| `phosphoric_demo_step` | Demo state-machine per-tick step. |
| `phosphoric_demo_render` | Demo per-frame render command emitter. |

These four symbols are pinned by the verify gate ([tools/verify/check_boot_phosphoric_only.sh](../tools/verify/check_boot_phosphoric_only.sh)) — the build manifest's `generated_symbols` field must equal the comma-separated list `efi_main,phosphoric_demo_init,phosphoric_demo_step,phosphoric_demo_render`.

Internal helper functions (e.g., `phosphoric_debug_puts`, `phosphoric_efi_puts`) are emitted but not exported. They are static within the boot image.

---

## 5. Helper Function Conventions

### 5.1 `phosphoric_debug_puts`

Writes a null-terminated byte string to QEMU's debug output port (`0x402`).

```
phosphoric_debug_puts:
    pushq %rbp
    movq %rsp, %rbp
    movq %rcx, %r8                  # r8 = string pointer
    movw $1026, %dx                 # 0x402 (debug port)
.Lphosphoric_debug_puts_next:
    movb (%r8), %al
    testb %al, %al
    je .Lphosphoric_debug_puts_done
    outb %al, %dx
    incq %r8
    jmp .Lphosphoric_debug_puts_next
.Lphosphoric_debug_puts_done:
    leave
    retq
```

Local labels use the `.L` prefix; the suffix is the function name plus a hyphenated descriptor. This convention is required by the PE/COFF writer's symbol-table parser.

### 5.2 `phosphoric_efi_puts`

Writes a UCS-2 string through `ConOut->OutputString` so it appears on the firmware framebuffer.

The function reads `SystemTable + 0x40` to find `ConOut`, then `ConOut + 0x08` to find the `OutputString` function pointer, then calls through it with Microsoft x64 calling convention.

```
phosphoric_efi_puts:
    pushq %rbp
    movq %rsp, %rbp
    subq $32, %rsp                  # shadow space for callee
    testq %rcx, %rcx
    je .Lphosphoric_efi_puts_done
    movq 0x40(%rcx), %rax           # rax = ConOut
    testq %rax, %rax
    je .Lphosphoric_efi_puts_done
    movq %rax, %rcx                 # rcx = this
    callq *0x08(%rax)               # OutputString
.Lphosphoric_efi_puts_done:
    addq $32, %rsp
    leave
    retq
```

The `0x40` and `0x08` offsets are pinned per UEFI 2.x. Changing them is a wire-protocol break.

---

## 6. State-Machine Functions

Every Phosphoric source function declared in the boot profile is emitted as one assembly function. The emitter follows these conventions:

### 6.1 Boolean state encoding

A `bool` is encoded as the low bit of a 64-bit register. Reading a bool field zero-extends:

```
movzwl %dx, %eax
cmpl $32, %eax
jne .L<label>_keep
```

Setting a bool flips the low bit:

```
movq %rcx, %rax
xorq $1, %rax
andq $1, %rax
```

### 6.2 Integer arithmetic

`u32` arithmetic uses 32-bit register forms (`%eax`, `%ecx`, etc.). `u64` uses 64-bit (`%rax`, `%rcx`). Sign-extension is explicit; the emitter never relies on implicit register width.

### 6.3 Control flow

Conditional branches always `cmpl` followed by a labeled `jne` / `je`. There are no unconditional `jmp` instructions except in loop bodies (none emitted in the current corpus) and in the helper function above.

Each branch target uses a deterministic local label naming scheme: `.L<function_name>_<descriptor>`. The descriptor is a single underscored word (e.g., `_keep`, `_done`, `_loop_top`).

---

## 7. Render Command Construction

The boot demo emits a bounded list of render commands per tick. The list is constructed in a caller-supplied buffer; the emitter does not allocate.

Each `RenderCommand` enum variant has a fixed encoding:

| Variant | Bytes | Layout |
|---|---|---|
| `FillRect(x, y, w, h, rgba)` | 1 + 4*5 = 21 | tag=0x01, then 5 LE u32 |
| `WritePixel(x, y, rgba)` | 1 + 4*3 = 13 | tag=0x02, then 3 LE u32 |
| `Present` | 1 | tag=0x03 |

The list header is two LE u16 (length + capacity) followed by the commands packed in declaration order.

---

## 8. Symbol Table

The emitter produces a flat symbol table at the end of the file:

```
.section .data
phosphoric_symtab:
    .quad efi_main
    .quad phosphoric_demo_init
    .quad phosphoric_demo_step
    .quad phosphoric_demo_render
```

Each entry is one 8-byte little-endian pointer. The PE/COFF writer reads this table to populate the export directory. The order is fixed; the verify gate enforces it.

---

## 9. Constants and Magic Numbers

The emitter uses these magic constants verbatim:

| Constant | Value | Meaning |
|---|---|---|
| QEMU debug port | `0x402` (1026) | OUT byte to this port writes to QEMU's debug log |
| ConOut offset | `0x40` (64) | `SystemTable + 0x40` = pointer to ConOut |
| OutputString offset | `0x08` (8) | `ConOut + 0x08` = OutputString function pointer |
| QEMU exit port | `0xF4` (244) | OUT byte to this port terminates QEMU |
| Shadow space | `32` | Microsoft x64 ABI mandate |
| Function alignment | `16` | `.p2align 4` |
| RenderCommand FillRect tag | `0x01` | |
| RenderCommand WritePixel tag | `0x02` | |
| RenderCommand Present tag | `0x03` | |

Changing any of these breaks byte equivalence. They are part of the boot ABI.

---

## 10. The Demo's Five Banners

The boot path emits five banner strings via `phosphoric_efi_puts` and `phosphoric_debug_puts`. The banners are part of the public verification gate ([tools/qemu-run/run_uefi_demo.sh](../tools/qemu-run/run_uefi_demo.sh) requires all five appear in the QEMU debug log):

```
phosphoric: entering generated boot-asm demo
phosphoric: generated boot-asm demo runtime active
phosphoric: event routed
phosphoric: redraw complete
phosphoric: demo complete
```

Each banner is a UCS-2 (UTF-16LE, 2 bytes per ASCII character) string with a trailing 0x0000 null terminator. The byte counts are deterministic; the boot image's layout is reproducible.

---

## 11. Determinism

The emitter is deterministic in two senses:

1. **Source-stable.** The same input source produces the same output bytes, regardless of when or where the emitter runs. Floating-point timestamps, RNG seeds, and PIDs are explicitly excluded from the encoder.
2. **Layout-stable.** Function entry alignments, label naming, shadow-space reservations, and section ordering are all computed deterministically from the source. There is no "format pass" that could vary the output.

The two stability properties together give byte-equivalent reproducibility — the same source on different builders produces the same output bytes.

---

## 12. Retirement Path

The Phosphoric-authored codegen ([compiler/codegen_boot.phos](../compiler/codegen_boot.phos)) is incrementally aligning to this spec. The retirement path is:

1. **Per-instruction byte-stream coverage** — each `BootIrInstr` variant emits its full argument-byte stream. Done.
2. **Helper function emission** — `phosphoric_debug_puts`, `phosphoric_efi_puts`, the symbol table, and the `.p2align 4` prologues. Pending.
3. **Calling convention wiring** — Microsoft x64 with shadow space. Pending.
4. **Per-instruction conformance** — the Phosphoric-emitted byte stream matches the shell emitter byte-for-byte on the conformance corpus. Pending.
5. **30-day green window** — the byte equivalence holds across CI runs.
6. **Shell emitter retirement** — the shell script moves to internal storage. The Phosphoric-authored backend becomes the spec.

The verify gate `phosphoric_lower_eq.phos` (host program) drives the comparison once both backends produce their output for the same input. Byte divergence is reported as a structured diagnostic; panic on divergence is forbidden.

---

## 13. What This Spec Is Not

- Not the language ABI. The language's calling convention, layout rules, and effect lattice live in [docs/abi.md](abi.md), [docs/language/memory_model.md](language/memory_model.md), and [docs/language/effect_lattice.toml](language/effect_lattice.toml).
- Not the boot IR spec. The intermediate representation between HIR and assembly is in [docs/BOOT_ABI_V1.md](BOOT_ABI_V1.md).
- Not a general assembly reference. This document captures only the narrow subset the boot emitter produces. New constructs are added by extending the conformance corpus, not by relaxing this spec.

---

## See Also

- [docs/BOOT_ABI_V1.md](BOOT_ABI_V1.md) — the IR spec that boot_asm_v1 lowers from
- [docs/abi.md](abi.md) — language-level ABI
- [docs/COMPILER.md](COMPILER.md) — compiler pipeline overview
- [tests/golden/boot_asm_v1_button_policy_golden.s](../tests/golden/boot_asm_v1_button_policy_golden.s) — the canonical golden output this spec describes
- [tools/phosphoric/emit_boot_demo_from_phos.sh](../tools/phosphoric/emit_boot_demo_from_phos.sh) — the legacy shell emitter (retiring)
- [compiler/codegen_boot.phos](../compiler/codegen_boot.phos) — the Phosphoric-authored backend
