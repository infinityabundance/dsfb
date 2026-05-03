# PFI0 — Phosphoric Forensic Incident container, version 0

## Apex statement

A `.pfi` file is a **closed evidentiary container**. Given the same
`.pfi` bytes, the classifier (`tools/phosphoric-host/phosphoric_drift.phos`)
emits the same verdict bytes — every time, on every host. There are no
timestamps, no host paths, no compression, no JSON, no UUIDs, no
optional fields. The format is byte-stable; deviation is doctrine
violation.

## Layout (PFI0)

A `.pfi` is a sequence of 32-byte aligned blocks. The total byte size
is `192 + 32 * (residual_count - 1)` for `residual_count >= 1`, or
`192` for `residual_count == 0`.

```
offset      block          contents
-------     -----          --------
0..32       header         magic="PFI0" (4) | residual_count u32 LE (4) | reserved [u8; 24]
32..64      manifest_hash  [u8; 32]
64..96      image_hash     [u8; 32]
96..128     stream_hash    [u8; 32]
128..160    record[0]      Residual (32 bytes per kernel/residual.phos §1)
160..160+32(N-1)  record[1..N-1]
160+32(N-1)..192+32(N-1)  footer  final_chain_hash [u8; 4] | reserved [u8; 28]
```

For a single-record `.pfi` (the common case, including the first
fixture): total bytes = `192`.

### Field semantics

| Field | Semantics |
|-------|-----------|
| `magic` | The four ASCII bytes `P`, `F`, `I`, `0`. Any other prefix is rejected as MALFORMED_CASE. |
| `residual_count` | Number of `Residual` records following `stream_hash`. u32 little-endian. Must equal the number of records actually present. |
| `manifest_hash` | SHA-256 of the declared task-seal manifest's canonical byte representation. The hash function is named in `kernel/residual.phos` adjacent comments; this header anchors *which* manifest produced the boundary. |
| `image_hash` | SHA-256 of the compiled task image bytes. Anchors *which* binary ran. |
| `stream_hash` | SHA-256 of the concatenation of the N residual records' bytes (each record exactly 32 bytes). Anchors *which* events were emitted. |
| `record[i]` | One 32-byte `Residual` record per `kernel/residual.phos` §1 (kind, arch_id, seq, cycle, payload, chain_hash + 2 bytes pad). |
| `final_chain_hash` | The `chain_hash` of the last residual in the stream. For `residual_count == 0` this is `[0; 4]`. |
| `reserved` bytes | Always zero. Non-zero reserved bytes are MALFORMED_CASE. |

### Invariants (court-enforced)

1. **Byte-stability.** Same inputs (manifest, image, residual stream)
   produce byte-identical `.pfi` bytes — no implementation-defined
   ordering, no host-dependent encoding.
2. **Stream coherence.** `stream_hash == SHA-256(record[0].bytes ||
   record[1].bytes || ... || record[N-1].bytes)`. Mismatch is
   MALFORMED_CASE.
3. **Chain coherence.** For each `i >= 1`, `record[i].chain_hash`
   equals `chain_step(record[i-1].chain_hash, record[i].event_bytes)`
   per `kernel/residual.phos` §`chain_step`. Mismatch is
   MALFORMED_CASE.
4. **Sequential integrity.** `record[i].seq == i + 1` (strictly
   monotonic, no gaps, starts at 1). Mismatch is MALFORMED_CASE.
5. **Kind closure.** `record[i].kind` is in `{1, 2, 3, 4, 5, 6, 7,
   0xFF}` per the `kernel/residual.phos` taxonomy. Other values are
   MALFORMED_CASE.
6. **Reserved zeroing.** All reserved byte regions are exactly zero.
7. **Total length.** `len(.pfi) == 192 + 32 * (residual_count - 1)`
   for `residual_count >= 1`; `len(.pfi) == 192` for
   `residual_count == 0`. Truncation or trailing bytes are
   MALFORMED_CASE.

## Why no JSON, no compression, no timestamps

Phosphoric is a court, not a logger. A court's evidence file is:
- **Byte-stable** so replay is byte-identical (a cryptographically-
  precise notion of "same case").
- **Self-describing only via fixed offsets** — the layout is published
  in this document, not embedded in the file. Adding self-description
  (TLV, JSON keys) creates ambiguity surfaces that drift over time.
- **Deterministic by construction** — no environmental nondeterminism
  (timestamps, host names, paths, UUIDs) can leak into evidence.
- **Adversarial-rejecting** — every field has a closed grammar; bad
  evidence produces `MALFORMED_CASE`, not "looks unusual".

Adding compression or optional fields would dissolve these properties.

## First fixture: `mmio_boundary_violation.pfi`

`tools/verify/fixtures/pfi/mmio_boundary_violation.pfi` is the
canonical first case file. It encodes the panel's recommended starter
case (R5 MMIO boundary violation):

- A task whose declared manifest authorizes MMIO range
  `0x1000..0x10FF`.
- The task touches address `0x1100` (one byte outside the upper
  declared bound).
- Runtime emits one `R5` `mmio_touch` residual record.
- The .pfi container wraps that record with manifest/image/stream
  hashes (sentinel patterns in this fixture; later fixtures will
  derive them from real artifacts).

Verified by `tools/verify/check_pfi_layout.sh` (Makefile target
`verify-pfi-layout`, wired into `verify-legendary`). The gate asserts
the 192-byte layout, the magic, the `residual_count`, the chain_hash
math (re-derived in awk), the kind closure, and the sequential
integrity.

## Future fixtures

Subsequent sessions will add:
- `bad_chain_hash.pfi` — chain_hash flipped → MALFORMED_CASE
- `seq_gap.pfi` — non-monotonic seq → MALFORMED_CASE
- `manifest_hash_mismatch.pfi` — manifest_hash doesn't match expected
- `truncated_record.pfi` — partial record at end
- `bad_kind.pfi` — kind outside the closed taxonomy

Each adversarial fixture asserts the classifier rejects the case with
a deterministic exit code, not a heuristic guess.

## Status as of 2026-04-30

This is the **first session of Stream C Milestone A** (Session 13,
2026-04-30). The PFI0 layout is now byte-stable; the first fixture is
in place; the gate is wired. Producer-side runtime emission of R5
records into a ring buffer is **still not implemented** — the .pfi
fixtures are hand-encoded for now. Sessions 14–17 close the remaining
Stream C milestones (B: R5 byte-layout fixture; C: classifier verdict
+ replay-idempotent; D: malformed-case rejection; E: no-silent-
authority gate).
