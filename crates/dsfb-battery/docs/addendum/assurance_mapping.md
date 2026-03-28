# Assurance Mapping Helper

Status: Supportive evidence only. Not an ASIL-D or DO-178C certification claim.

| Assurance topic | Current DSFB evidence | Crate component | Status |
|---|---|---|---|
| Determinism | Fixed-order arithmetic, repeated-run determinism helper, finite grammar-state set | `src/math.rs`, `src/detection.rs`, compliance/addendum helpers | Partial |
| Non-interference | Read-only, advisory interface contract and shadow-mode notes | `src/audit.rs`, `src/integration.rs` | Partial |
| Auditability | Event-level audit trace, hashes, reason codes | `src/audit.rs` | Partial |
| Bounded interface contract | Narrow FFI plus ICD | `src/ffi.rs`, `docs/addendum/icd.md` | Partial |
| Formal proof harness availability | Kani scaffolds for core helper invariants | `formal/kani/` | Partial |
| Testability | Existing crate tests, addendum tests, compliance tests | `src/*` tests | Partial |
| Freedom from production-path interference | Addendum outputs isolated under `outputs/addendum/...` | addendum/compliance helpers | Supported |

What remains outside current evidence:

- qualified toolchain evidence
- certification package process evidence
- fielded hardware/software integration evidence
- hardware fault-tolerance qualification
