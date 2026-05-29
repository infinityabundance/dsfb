# cargo-geiger — `unsafe` usage audit

`cargo geiger` counts `unsafe` functions / expressions / impls / traits / methods across the dependency tree.
Run from the `edge` crate (which pulls in `atlas`, `corpus`, `core` as path deps). Raw output:
`cargo-geiger-edge.txt`; cleaned count tree: `cargo-geiger-tree.txt`.

## First-party posture (ground truth, from source — the load-bearing result)

| Crate | `#![forbid(unsafe_code)]` | first-party `unsafe` |
|---|---|---|
| edge                  | **yes** | **0** |
| atlas                 | **yes** | **0** |
| corpus                | **yes** | **0** |
| core                  | **yes** | **0** |
| dsfb-densor-runtime    | **yes** | **0** |
| cuda                  | no | 5 (CUDA FFI boundary only) |
| wasm                  | `#![deny(unsafe_code)]` + 1 audited `#[allow]` block | 1 (linear-memory FFI marshalling) |

**Five of seven crates forbid `unsafe` entirely (zero `unsafe`).** All first-party `unsafe` is confined to two
declared FFI boundaries: the CUDA host↔device interface (`cuda`) and one audited linear-memory marshalling
block in the `wasm` shell (exercised under Miri — see `audit/miri/`). (`dsfb-densor-runtime`, the new execution
substrate, joins as a fifth `#![forbid(unsafe_code)]` crate; its posture is taken from source — `src/lib.rs:25`
plus a zero-`unsafe` grep — since geiger is run from `edge`, which does not depend on it.)

## Dependency tree
The `!` flags in `cargo-geiger-edge.txt` are **dependencies** that use `unsafe` internally — `sha2`, `memchr`,
`hashbrown`, `itoa`, `winnow`, etc. — which is normal and expected for cryptographic and SIMD-optimised crates.
The dependency surface is small (serde/serde_json/toml/csv/thiserror + transitive).

## Honest caveat
cargo-geiger emitted "Failed to match … package" warnings for the path-dependency first-party crates (a known
cargo-geiger limitation with workspace path deps); its per-crate counters for those packages are therefore
unreliable, which is why the first-party posture above is taken directly from the source
(`#![forbid(unsafe_code)]` attributes + an `unsafe`-keyword grep), not from geiger's matcher.
