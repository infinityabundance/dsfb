## DSFB-Gray: Deterministic Rust Crate Auditing and Assurance Engine

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
![Unsafe](https://img.shields.io/badge/unsafe-deny-brightgreen)
![Mode](https://img.shields.io/badge/runtime-read--only-blue)
![Scan](https://img.shields.io/badge/attestation-SARIF%20%2B%20DSSE-orange)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-gray/notebooks/dsfb_gray_crates_audit.ipynb)

**DSFB-Gray is a deterministic Rust crate auditing system that performs structural code-quality interpretation, emits assurance scores, and produces machine-verifiable attestation artifacts.

The crate operates as a read-only audit layer over Rust source trees, generating reproducible outputs intended for code review, CI integration, and supply-chain traceability. It does not claim certification or compliance.**

The crate exposes a unified surface with a clearly defined primary interface:

- Static scanner (primary): deterministic Rust crate auditing, assurance scoring, and artifact generation
- Attestation layer: SARIF / in-toto / DSSE export for portable audit artifacts
- Runtime observer (secondary): retained DSFB structural interpretation components
- Evaluation harness: deterministic artifact regeneration and paper support

The project scope is broad by design, but the evidence model is intentionally narrow: only claims that can be regenerated from code and artifacts in this repository should be treated as authoritative. Everything else is descriptive context, not empirical evidence.

**Evaluation Guidance.** This repository should be evaluated primarily as a deterministic Rust crate auditing system. The static scanner defines the primary interface. Other DSFB components present in the codebase are retained framework elements and are not required to use or evaluate the audit pipeline.




## What This Does NOT Do

- Does not modify the observed system (read-only, immutable references only)
- Does not replace Prometheus, Grafana, tracing, or any monitoring stack
- Does not require ML training or probabilistic models
- Does not use `unsafe` (enforced: `#![deny(unsafe_code)]`)
- Does not certify crates or systems by itself
- Does not infer live gray failures from static source code alone


## What This Does

DSFB treats a Rust crate as a structured audit surface:

`source structure -> static findings -> assurance signals -> audit trace -> attestation`

The crate also retains a broader DSFB runtime observer framework, which can consume telemetry streams and produce structural interpretations. In this repository, that runtime path is secondary and is not required to use or evaluate the static audit pipeline.

## Claims and Evidence Discipline

- Public metrics in this repository are regenerated from code, not maintained by hand.
- Static scanner findings are source-visible structural proxies, not operational proof.
- Runtime results are deterministic harness results, not production-validation claims.
- The paper and README should track the same evidence base as `data/evaluation_results.txt`.

## Quick Start

Single command:

```bash
cargo run --bin dsfb-scan-crate tokio
```

Other Commands:

```bash
cargo test
cargo run --bin dsfb-demo
cargo run --bin dsfb-regenerate-public-artifacts
cargo check --no-default-features
```

Crate audits produced by `dsfb-scan-crate` are written into timestamped run
folders under `output-dsfb-gray/`, for example:

```text
output-dsfb-gray/dsfb-gray-2026-04-14T01-23-45Z/
  tokio_scan.txt
  tokio_scan.sarif.json
  tokio_scan.intoto.json
  tokio_scan.dsse.json
```

The scanner still prints the human-readable report to stdout, and also writes
the same `.txt` report plus the structured artifacts into the run directory.
DSFB now emits one canonical broad audit. Domain and standards interpretations
appear as conclusion lenses at the end of the report rather than as primary scan
profiles.

## Why Run This

DSFB-Gray is useful when you need a deterministic, inspectable audit of a Rust crate that can be:

- stored as a reproducible artifact
- compared across versions
- integrated into CI without introducing non-deterministic signals
- exported into standard formats (SARIF, in-toto, DSSE)

It is not intended to replace existing tools, but to provide a coherent, reproducible audit surface across them.

DSFB-Gray is designed to operate alongside existing tools such as Clippy, cargo-audit, and external static analyzers, aggregating and structuring their signals into a deterministic audit surface rather than replacing them.

## Reproducibility: Auditing a Real Rust Crate

This section provides a concrete procedure for reproducing a DSFB audit on a real Rust crate. The example uses the widely adopted asynchronous runtime [`tokio`](https://crates.io/crates/tokio).

### Environment Requirements

- Rust toolchain (stable) installed via `rustup`
- Network access for initial crate retrieval
- Local filesystem access for artifact generation

---

### Direct Scan from crates.io (Recommended)

After installing or cloning `dsfb-gray`, a crate can be scanned directly by name:

```bash
cargo run --bin dsfb-scan-crate tokio
```
---

### Alternative: Manual Local Path Scan

For full control over the exact source version:

```bash
cargo fetch
```

Locate the crate under:

```bash
$HOME/.cargo/registry/src/
```

Then run:
```bash
cargo run --bin dsfb-scan-crate -- \
    --path /path/to/tokio-1.x.x/
```

### Outputs

The audit produces:

- Deterministic assurance score and subscores
- Structured findings with reason codes
- Machine-readable artifacts (SARIF, in-toto, DSSE)
- Human-readable reports (text/CSV)

Artifacts are written to:
```md
output-dsfb-gray/<timestamp>/
```

### Colab Notebook Execution

A companion Colab notebook:

- installs the crate from scratch
- accepts a crates.io crate name
- runs the audit
- displays results inline
- exports a ZIP of all artifacts

### Determinism Note

Repeated execution on the same crate version and configuration produces identical outputs.

This holds under:

- identical crate version
- identical DSFB configuration
- identical Rust toolchain

No claim is made for invariance across toolchain or crate changes.





## Integrated Stack

- Static scanner (primary interface): deterministic crate auditing and assurance scoring
- Attestation layer: portable audit artifact export (SARIF, in-toto, DSSE)
- Public-artifact generator: reproducible audit and paper outputs
- Runtime observer (secondary, retained): DSFB structural interpretation pipeline

## Runtime Evaluation (Supporting, Non-Primary)

These results correspond to the retained DSFB runtime observer evaluation harness and are included as supporting evidence of the broader framework. They are not the primary evaluation surface of the crate’s static auditing functionality.

<!-- DSFB:README_RESULTS:BEGIN -->
| Gray Failure Scenario | Detection Delay | Lead Time | False Alarms |
|----------------------|-----------------|-----------|--------------|
| Clock Drift | 7 steps | 143 steps | **0** |
| Partial Partition | 2 steps | 158 steps | **0** |
| Channel Backpressure | 9 steps | 161 steps | **0** |
| Async Starvation | pre-injection | 195 steps | **1** |

Current metrics are generated by `cargo run --bin dsfb-regenerate-public-artifacts`. The current recommended configuration detects 4/4 primary scenarios, and 1 primary scenario(s) show a pre-injection anomaly.
<!-- DSFB:README_RESULTS:END -->

Generated outputs are written to `data/evaluation_results.txt`, `data/demo-output.txt`, the scenario CSV files in `data/`, and the generated snippets in `docs/generated/` and `paper/generated/`.

<!-- DSFB:EVIDENCE_LEDGER:BEGIN -->
## Evidence Ledger

Every public-facing numeric claim in this repository should map to one command, one artifact, or one generated section.

| Claim Surface | Generated From | Artifact |
|---------------|----------------|----------|
| README results table | `cargo run --bin dsfb-regenerate-public-artifacts` | `docs/generated/README_RESULTS.md` |
| Full evaluation narrative | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/evaluation_results.txt` |
| Demo output | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/demo-output.txt` |
| Sensitivity sweep table | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/sensitivity_sweep.csv` |
| Scenario CSV: Clock Drift | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/clock_drift.csv` |
| Scenario CSV: Partial Partition | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/partial_partition.csv` |
| Scenario CSV: Channel Backpressure | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/channel_backpressure.csv` |
| Scenario CSV: Async Starvation | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/async_starvation.csv` |
| Paper TeX results table | `cargo run --bin dsfb-regenerate-public-artifacts` | `paper/generated/results_summary.tex` |
| Audit contract summary | `cargo run --bin dsfb-regenerate-public-artifacts` | `docs/generated/AUDIT_CONTRACT.md` |
| Paper TeX audit contract | `cargo run --bin dsfb-regenerate-public-artifacts` | `paper/generated/audit_contract.tex` |
| Claim ledger | `cargo run --bin dsfb-regenerate-public-artifacts` | `docs/generated/CLAIM_LEDGER.md` |
<!-- DSFB:EVIDENCE_LEDGER:END -->

## Assurance Scoring

The crate scanner now emits a locked percentage-based assurance score using the
method identifier `dsfb-assurance-score-v1`.

The score is a broad improvement and review-readiness target. It is not a
compliance certification.

The scoring specification is fixed in
[`docs/AUDIT_SCORING_LOCKED.md`](docs/AUDIT_SCORING_LOCKED.md). Any future
change to weights, thresholds, checkpoint sets, or score bands must create a
new method identifier and a new locked specification.

The next-phase implementation plan for hazard interpretation lenses, runtime attestation
binding, formal property bridges, and refactor guidance is in
[`docs/IMPLEMENTATION_ROADMAP.md`](docs/IMPLEMENTATION_ROADMAP.md).

Legacy root-level `*_scan.*` artifacts are automatically migrated into
`output-dsfb-gray/` the next time the scanner runs.

## Public Artifact Regeneration

`cargo run --bin dsfb-regenerate-public-artifacts` is the canonical regeneration path for:

- `data/evaluation_results.txt`
- `data/demo-output.txt`
- scenario CSVs under `data/`
- `docs/generated/*`
- `paper/generated/*`
- the generated README and paper claim/result sections

## Project Structure

```
dsfb-gray/
├── CHANGELOG.md
├── data/                  # Deterministic evaluation artifacts (regenerated)
│   ├── evaluation_results.txt
│   ├── demo-output.txt
│   ├── sensitivity_sweep.csv
│   └── <scenario>.csv
├── docs/
│   ├── AUDIT_SCORING_LOCKED.md # Locked scoring specification
│   ├── IMPLEMENTATION_ROADMAP.md # Next-phase feature roadmap
│   ├── INTEGRATION_GUIDE.md # Runtime / scanner / attestation integration
│   ├── CAPABILITY_LADDER.md # Broad-scope capability framing
│   └── SCAN_TRIAGE.md # How to read noisy vs useful scan findings
├── examples/
│   ├── telemetry_adapter_integration.rs
│   └── scan_to_runtime_prior.rs
├── paper/
│   ├── paper.md           # Markdown paper with generated sections
│   ├── paper.tex          # TeX paper with generated tables
│   └── generated/         # Generated paper fragments
├── scripts/
│   └── regenerate_public_surface.sh
├── src/
│   ├── lib.rs             # Unified public API + feature layout
│   ├── adapter.rs         # TelemetryAdapter and integration bridge
│   ├── residual.rs        # ResidualSign estimator (r, ω, α)
│   ├── envelope.rs        # Admissibility envelopes
│   ├── grammar.rs         # Grammar state machine with hysteresis
│   ├── heuristics.rs      # Heuristics bank + bounded static priors
│   ├── episode.rs         # Operator-facing episode objects
│   ├── observer.rs        # Runtime observer + reason evidence
│   ├── regime.rs          # Workload phase classifier
│   ├── audit.rs           # Deterministic audit trace
│   ├── inject.rs          # Fault injection scenarios
│   ├── report.rs          # Plain-text and CSV report generation
│   ├── evaluation.rs      # Canonical public-evidence generator
│   ├── scan.rs            # Static scanner + attestation export
│   └── bin/
│       ├── dsfb-demo.rs
│       ├── dsfb-regenerate-public-artifacts.rs
│       └── dsfb-scan-crate.rs
└── output-dsfb-gray/     # Timestamped scan runs
```

## The Heuristics Bank

12 typed entries encoding patterns that experienced Rust engineers recognize implicitly:

| ID | What It Detects | Rust Provenance |
|----|----------------|-----------------|
| H-ALLOC-01 | Allocation jitter at capacity doubling | `Vec<T>` in hot loop |
| H-LOCK-01 | RwLock contention escalation | Read→write transition |
| H-RAFT-01 | Consensus heartbeat degradation | openraft election timeout |
| H-ASYNC-01 | Async runtime starvation | Blocking in async context |
| H-TCP-01 | Partial partition signature | Selective packet loss |
| H-CHAN-01 | Channel backpressure onset | `mpsc` bounded channel |
| H-CLOCK-01 | Clock source divergence | TSC vs HPET discrepancy |
| H-THRU-01 | Throughput degradation | IO scheduler starvation |
| H-SERDE-01 | Serialization drift | Payload growth / schema migration |
| H-GRPC-01 | Flow control exhaustion | tonic h2 window starvation |
| H-DNS-01 | DNS resolution drift | Cache poisoning |
| H-ERR-01 | Error rate escalation | Connection pool exhaustion |

## Non-Interference Contract (v1.0)

All inputs accepted as `&ResidualSample` (immutable references). No mutable reference to any upstream system component is created. If the observer layer is removed from the dependency tree, the observed system compiles and behaves identically.

## API Direction

The library surface is being hardened around four public concepts:

- `TelemetryAdapter<T>` for translating application telemetry into `ResidualSample`
- `StaticPriorSet` for bounded scan-to-runtime structural biasing
- `ReasonEvidence` for explaining why a reason code fired
- one canonical broad audit plus attestation exports for CI and supply-chain workflows

The goal is not to narrow the project. It is to make the broad stack legible and reusable as one coherent library.

Release-note discipline lives in [`CHANGELOG.md`](CHANGELOG.md).

## Related Work

This is a domain instantiation of the DSFB framework. Other instantiations:
- [Gas Turbine Health Monitoring](https://doi.org/10.5281/zenodo.19498878) — C-MAPSS evaluation
- [Semiconductor Process Control](https://crates.io/crates/dsfb-semiconductor) — SECOM dataset
- [Battery Health Monitoring](https://doi.org/10.5281/zenodo.19176473) — NASA PCoE

## Citation

```bibtex
@software{debeer2026dsfbgray,
  author    = {de Beer, Riaan},
  title     = {{DSFB Structural Semiotics Engine for Deterministic Rust Crate
               Auditing: A Non-Intrusive Deterministic Augmentation Layer for
               Structural Code Quality Interpretation and Certification
               Compliance Estimation}},
  year      = {2026},
  version   = {1.0},
  publisher = {Zenodo},
  doi       = {10.5281/zenodo.19600872},
  url       = {https://doi.org/10.5281/zenodo.19600872}
}
```

## License

The theoretical framework, formal constructions, and supervisory methods described herein constitute proprietary Background IP of Invariant Forge LLC (Delaware LLC No.\ 10529072), with prior art established by this publication and earlier Zenodo DOI publications by the same author. Commercial deployment requires a separate written license. Reference implementations are released under Apache~2.0. 

Licensing:
licensing@invariantforge.net 

[LICENSE](LICENSE)

## Author

**Riaan de Beer** — Chief Research Advisor, [Invariant Forge LLC](https://invariantforge.net)
ORCID: [0009-0006-1155-027X](https://orcid.org/0009-0006-1155-027X)
