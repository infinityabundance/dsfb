# MOSA Compatibility Note

Status: Supportive mapping only. No MOSA certification claim is made.

The crate already decomposes into replaceable boundaries that support a modular-open-systems interpretation:

| Boundary | Crate component | Role | Replaceable without changing production outputs? |
|---|---|---|---|
| Signal ingestion | `load_b0005_csv`, external residual helpers | Acquires capacity or upstream residual inputs | Yes, if input contract is preserved |
| Residual processing | `src/math.rs` | Residual, drift, slew, envelope computation | Yes |
| Grammar engine | `src/detection.rs` | Persistence logic, state evaluation, reason code assignment | Yes |
| Heuristics bank | `config/heuristics_bank_v1.json`, `src/heuristics.rs` | Versioned interpretive notes | Yes |
| Audit/reporting | `src/audit.rs`, `src/export.rs`, `src/plotting.rs` | Host-side artifact generation | Yes |
| FFI/wrapper layer | `src/ffi.rs`, `include/dsfb_battery_ffi.h`, `wrappers/` | Integration surface for external hosts | Yes |

Supporting points:

- The core DSFB engine is separable from host-side reporting through the current `std` / `no_std + alloc` split.
- The addendum layer introduces separate wrappers and mappings rather than changing the paper-facing path.
- The component map at `docs/addendum/mosa_component_map.json` gives a machine-readable view of the current modular boundaries.
