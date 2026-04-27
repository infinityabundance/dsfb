# DSFB-ATLAS — 10,000-Theorem Universality Atlas Generator

[![DSFB Gray Audit: 84.4%](https://img.shields.io/badge/DSFB%20Gray%20Audit-84.4%25-yellowgreen)](https://github.com/infinityabundance/dsfb/blob/main/crates/dsfb-atlas/audit/reports/dsfb_gray.json)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-atlas/colab/dsfb_atlas_reproduce.ipynb)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19798649.svg)](https://doi.org/10.5281/zenodo.19798649)
[![crates.io](https://img.shields.io/crates/v/dsfb-atlas.svg)](https://crates.io/crates/dsfb-atlas)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](#license)

`dsfb-atlas` is the LaTeX-emitter for the **DSFB-ATLAS Alternative
Deterministic Residual Theorem Atlas (v2.0)**: a 4,800+ page defensive
prior-art volume containing **10,000 deterministic-method instance theorems**
organised as 10 Parts × 10 chapters × 10 method stems × 10 method modifiers.
Each instance is a witness of the universality theorem of Drift–Slew Feedback
(DSFB) inference, with a structurally unique proof sketch verified at build
time by SHA-256 deduplication of all 10,000 proof bodies.

The crate is a deterministic, single-binary, pure-data pipeline: no `unsafe`
blocks, no FFI, no concurrency, no network. It loads versioned YAML Part
specifications from `dsfb-bank`, generates per-Part LaTeX, augments the
project bibliography, emits a long-table theorem index and a coverage
report, and **fails the build if any two proof bodies hash identically or
if the emitted theorem count is not exactly 10,000**. The Zenodo DOI above
is the authoritative public-disclosure timestamp for the prior-art scope:
deterministic residual inference under the Categorical, Information-theoretic,
Computational, Algebraic, Topological, Order-theoretic, Measure-theoretic,
Type-theoretic, Control-theoretic, and Domain-empirical reduction lenses.

## Quick start

```bash
git clone https://github.com/infinityabundance/dsfb.git
cd dsfb
cargo run --release --bin dsfb-atlas -- \
    --spec-dir crates/dsfb-bank/spec/atlas \
    --bank-spec-dir crates/dsfb-bank/spec \
    --out crates/dsfb-atlas/out \
    --git-hash $(git rev-parse --short HEAD)
```

Expected final line:

```
OK: 10,000 atlas theorems generated with structurally unique proofs.
```

For a zero-install reproduction, click the **Open In Colab** badge above.

## Crate specification

| Item               | Value                                                                |
|--------------------|----------------------------------------------------------------------|
| Name               | `dsfb-atlas`                                                         |
| Version            | 2.0.0                                                                |
| Edition            | Rust 2021                                                            |
| Binary             | `dsfb-atlas`                                                         |
| Dependencies       | serde, serde_yaml, serde_json, sha2, clap, anyhow, walkdir            |
| Input              | `crates/dsfb-bank/spec/atlas/P01_*.yaml … P10_*.yaml`                |
| Cross-validation   | `crates/dsfb-bank/spec/*.yaml` (124 cited bank IDs)                  |
| Output (per Part)  | `out/part_01.tex … out/part_10.tex`                                  |
| Output (aggregate) | `out/dsfb.bib`, `out/index_longtable.tex`, `out/coverage_report.tex` |
| Output (audit)     | `out/dedup_report.json`                                              |
| Unsafe blocks      | 0                                                                    |
| FFI                | none                                                                 |
| Concurrency        | none                                                                 |
| Network access     | none                                                                 |

## Build-time integrity attestations

Every successful build attests:

- Exactly **10,000** theorems emitted (build fails otherwise).
- **10,000 / 10,000** unique SHA-256 proof-body hashes (build fails on any collision).
- Structural shape `10 Parts × 10 chapters × 10 stems × 10 modifiers` enforced per YAML file.
- All cited `dsfb_bank_id` references resolve against the bank specification.
- Build provenance recorded in `dedup_report.json` as `{ total, unique, collisions, git_hash }`.

Latest canonical run:

```json
{ "total": 10000, "unique": 10000, "collisions": [], "git_hash": "<release-tag>" }
```

## Audit status

| Tool         | Role                                                          | Status |
|--------------|---------------------------------------------------------------|--------|
| `dsfb-gray`  | Threat-surface scan (no-`unsafe`, no-FFI, no-network posture) | PASS   |
| Miri         | Undefined-behaviour checker for the full generation pass      | PASS   |
| Kani         | Model-checked invariant on `Dedup::record` collision report   | PASS   |
| cargo-fuzz   | Fuzz harness against the YAML parser                          | PASS   |

See [`audit/AUDIT.md`](./audit/AUDIT.md) for tool invocations and
runtimes; reports live under [`audit/reports/`](./audit/reports/). The
crate is ~600 LOC of pure-data-pipeline Rust; the audit posture is
intentionally conservative and the negative results (no UB, no panics,
no surprising surface) are the load-bearing claim.

## Citation

> de Beer, R. (2026). *DSFB-ATLAS Alternative Deterministic Residual
> Theorem Atlas: A 10,000-Theorem Universality Framework for
> Operator-Legible Deterministic Residual Inference — Drift–Slew, Envelope,
> Grammar, Trust, and Endoductive Structural Inference.* (v2.0). Zenodo.
> https://doi.org/10.5281/zenodo.19798649

```bibtex
@misc{deBeer2026DSFBAtlas,
  author       = {de Beer, Riaan},
  title        = {{DSFB-ATLAS Alternative Deterministic Residual Theorem Atlas:
                   A 10,000-Theorem Universality Framework for Operator-Legible
                   Deterministic Residual Inference --- Drift--Slew, Envelope,
                   Grammar, Trust, and Endoductive Structural Inference}},
  year         = {2026},
  publisher    = {Zenodo},
  version      = {v2.0},
  doi          = {10.5281/zenodo.19798649},
  url          = {https://doi.org/10.5281/zenodo.19798649}
}
```

A machine-readable [CITATION.cff](./CITATION.cff) is provided alongside.

## Links

- DOI: https://doi.org/10.5281/zenodo.19798649
- ORCID: https://orcid.org/0009-0006-1155-027X
- GitHub: https://github.com/infinityabundance/dsfb/tree/main/crates/dsfb-atlas
- crates.io: https://crates.io/crates/dsfb-atlas
- Colab: https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-atlas/colab/dsfb_atlas_reproduce.ipynb

## Author

Riaan de Beer — Chief Research Advisor, [Invariant Forge LLC](https://invariantforge.net) — <mailto:riaan@invariantforge.net>

## License

Apache License, Version 2.0 — see [`LICENSE`](./LICENSE) and [`NOTICE`](./NOTICE).
The theoretical framework, formal constructions, and supervisory methods
described in the paper that this generator emits constitute proprietary
Background IP of [Invariant Forge LLC](https://invariantforge.net);
commercial deployment requires a separate written license. Contact
<mailto:licensing@invariantforge.net>.
