# Changelog

All notable changes to `dsfb-database` are documented here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The deterministic-replay invariants (residual + episode SHA256 fingerprints
pinned in `tests/deterministic_replay.rs`) are part of the public contract:
any change that intentionally alters them is a MINOR-version event at minimum
and must update both this file and the corresponding paper section
(§8 Reproducibility, §13).

## [Unreleased]

### Added
- PostgreSQL `pg_stat_statements` adapter (`src/adapters/postgres.rs`):
  CSV ingest of the v14+ schema, emits `plan_regression` and `cardinality`
  residual channels. New CLI subcommand `dsfb-database ingest --engine postgres
  --csv <path>`. Bundled redacted sample under
  `examples/data/pg_stat_statements_sample.csv` (50 rows, md5-hashed query
  ids, no real query text). Example `examples/postgres_ingest.rs` pins both
  the residual and episode fingerprints so a silent adapter drift is caught
  in CI.
- Snowset adapter `load()` implementation: parses the cloud-warehouse query
  log into workload-phase residuals via per-window digest-mix entropy.
  Real-data figure `figs/snowset.workload_phase.png` and the §6 honesty
  paragraph distinguishing what was exercised on real data versus
  exemplar-only.
- §11 per-engine motif-deployability matrix (PostgreSQL / SQL Server /
  MySQL / Oracle × five motifs) with footnote-level honesty about what
  each engine's default telemetry does and does not expose.
- §11 deployment topology TikZ figure (engine → telemetry export → adapter
  → motif engine → episode stream → operator dashboard).
- §11 security/privacy posture paragraph: explicit about what the SHA256
  fingerprint does (replay integrity of stored residual bytes) and does
  not (workload integrity, query-text privacy, cryptographic proof).
- §5 worked operator narrative: five-step walkthrough of a `cache_collapse`
  episode from residual sample to operator action.
- §5 drift–slew phase portrait figure (`r_k` vs `s_k` with envelope
  classification, explicitly not framed as a strange attractor).
- §13 script-to-result mapping table: every figure / table number → the
  exact CLI command + output CSV path.
- §13 Direction D — Adversarial residual robustness (hypothesis only;
  the future-work non-claim tcolorbox makes the absence of empirical
  validation explicit).
- §4 control-theory bridge paragraph: frames the system as a structural
  state observer in the engineering sense, with explicit non-claims
  about loop-closure and optimiser-feedback-freeness.
- `tests/stress_sweep.rs`: pins (a) sweep determinism across re-runs at
  three scales and (b) baseline per-motif episode counts.
- `tests/spec_validation.rs`: parses `spec/{motifs,perturbations,wizard_concordance}.yaml`
  and asserts cross-references resolve to real `MotifClass` variants.
- `tests/deterministic_replay.rs::paper_ceb_episode_fingerprint_is_pinned`:
  pins the CEB exemplar episode fingerprint (mirrors the existing TPC-DS pin).
- `tests/non_claim_lock.rs::paper_non_claims_section_matches_crate_strings`:
  parses the §10 Non-Claims block from `paper/dsfb-database.tex` and asserts
  byte-equality with the crate's `NON_CLAIMS` array.
- `Cargo.toml`: `authors` field.
- Root `.gitignore` and crate-local `.gitignore` (Cargo, output dirs,
  LaTeX build artefacts, editor junk).
- This `CHANGELOG.md`.

### Changed
- `src/adapters/postgres.rs`, `src/adapters/sqlshare.rs`: HashMap iteration
  ordered explicitly so residual streams are bytewise reproducible.
- `spec/wizard_concordance.yaml`: three lines requoted so the YAML parses
  cleanly under `serde_yaml`.

### Fixed
- `src/residual/mod.rs`: rustdoc broken intra-doc link (`paneldiscussion.txt`
  was being interpreted as a doc-link target).
- Six clippy warnings under `--release --all-targets -- -D warnings`
  (empty-string `writeln!`, identical if-blocks, loop-index-as-iterator,
  manual-clamp pattern, useless `vec!` in tests, explicit auto-deref).

## [0.1.0] — 2026-04-XX (initial Phase-I drop)

### Added
- Five typed residual channels (`plan_regression`, `cardinality`,
  `contention`, `cache_io`, `workload_phase`) and a deterministic
  `ResidualStream` with SHA256 fingerprint.
- Five motif state machines (`PlanRegressionOnset`,
  `CardinalityMismatchRegime`, `ContentionRamp`, `CacheCollapse`,
  `WorkloadPhaseTransition`) with drift (EMA) + slew (instantaneous)
  envelope classification and a Stable / InEpisode / Recovering FSM.
- TPC-DS-shaped controlled perturbation harness with five labelled
  injected windows; the published baseline (seed=42, scale=1.0)
  produces the headline-pinned residual and episode fingerprints.
- Five dataset adapters (Snowset, SQLShare, CEB, JOB, TPC-DS) with
  explicit `[exemplar]` labelling for the synthetic-but-realistic
  generators.
- CLI subcommands: `reproduce`, `exemplar`, `replay-check`, `elasticity`.
- Paper (`paper/dsfb-database.tex`) covering motivation, DSFB background,
  drift–slew anatomy, perturbation harness, evaluation, limitations,
  non-claims, deployment path, future work, reproducibility.
- 23-item limitations enumeration; five non-claims pinned by
  `tests/non_claim_lock.rs`.
- Threshold elasticity sweep (±20 %), stress sweep across perturbation
  magnitudes, noise-reduction funnel, time-to-detect matrix, throughput
  micro-benchmark.
- `scripts/reproduce_paper.sh`: single command to rebuild every figure
  and table and the paper PDF.
