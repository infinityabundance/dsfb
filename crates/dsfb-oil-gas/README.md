# DSFB Oil & Gas

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-oil-gas/notebook/dsfb_oil_gas.ipynb)

**Drift–Slew Fusion Bootstrap (DSFB): Structural Residual Semantics for Upstream and Midstream Oil and Gas Systems**

A deterministic, read-only residual structuring framework implemented in Rust, with a companion LaTeX monograph paper and a reproducible Jupyter notebook (`notebook/dsfb_oil_gas.ipynb`, regenerable via `scripts/gen_notebook.py`).

---

## What DSFB Does

DSFB accepts a stream of (expected, observed) signal pairs from any oil-and-gas sensor channel.  It decomposes the residual `r_k = observed_k − expected_k` into three components:

| Component | Symbol | Description |
|-----------|--------|-------------|
| Residual | r\_k | Signed instantaneous deviation |
| Drift | δ\_k^(w) | Sliding-window mean of residuals |
| Slew | σ\_k | First-order finite-difference rate |

These three values are mapped through a calibrated **admissibility envelope** into a 7-token **operational grammar**:

| Token | Trigger condition | Typical physics |
|-------|-------------------|--------------------|
| `Nominal` | All components interior | Steady-state operation |
| `DriftAccum` | Drift exceeds bound | Wax build-up, bit wear, scale deposit |
| `SlewSpike` | Slew rate exceeds bound | Pigging, stick-slip, gas lock, valve actuation |
| `EnvViolation` | Residual outside envelope | Gross sensor offset, large transient |
| `BoundaryGrazing` | Residual in grazing band | Near-miss, transition approach |
| `Recovery` | Previous non-Nominal → interior | Return to steady state |
| `Compound` | Two or more simultaneous | Complex interaction |

Contiguous runs of the same token are compressed into **episodes**.  The output is a deterministic event log.

---

## What DSFB Does NOT Do

- **Does not predict failures.** It structures residuals. It makes no prognostic claim.
- **Does not replace RTTM, Kalman filters, MPC, SPC, or any existing system.** It is an observer layer.
- **Does not write to any upstream system.** No SCADA tag, alarm limit, historian archive, setpoint, or control variable is modified.
- **Does not require machine learning**, model training, or labeled data.
- **Does not provide sensor validation.** If the sensor is wrong, DSFB structures a wrong signal deterministically.
- **Does not work below Nyquist.**

---

## Non-Intrusive Integration Contract

DSFB is removable with **zero upstream impact**.

```
Physical Plant → Historian / SCADA Tag ← existing control logic
                         │
                   (read-only tap)      ← DSFB reads here only
                         │
                    DSFB Engine → Deterministic Event Log
```

The `ReadOnlySlice<T>` type enforces this at the API level: no mutable reference to upstream data is ever exposed.

---

## Crate Structure

```
src/
├── types.rs        — Core types (ResidualSample, AdmissibilityEnvelope, GrammarState, Episode)
├── residual.rs     — DriftEstimator (ring buffer, O(1)), SlewEstimator, ResidualProcessor
├── envelope.rs     — CoordClass, EnvelopeEval, evaluate()
├── grammar.rs      — GrammarClassifier FSM, DeterministicDsfb engine
├── events.rs       — aggregate_episodes(), summarise(), CSV/JSONL export
├── pipeline.rs     — PipelineFrame (flow balance, pressure differential)
├── drilling.rs     — DrillingFrame (torque, WOB, RPM)
├── rotating.rs     — RotatingFrame (ESP/compressor head, vibration)
├── subsea.rs       — SubseaFrame (actuation pressure, valve command)
├── integration.rs  — ReadOnlySlice, process_read_only, NonIntrusiveGuarantee
├── report.rs       — format_summary(), noise_compression_ratio()
├── loaders.rs      — CSV loaders for all four domain frames
├── error.rs        — DsfbError
└── lib.rs          — Public API re-exports
```

---

## Quick Start

```bash
# Build the crate
cargo build

# Run tests (unit + integration)
cargo test

# Run Kani formal verification proofs (requires `cargo install --locked kani-verifier && cargo kani setup`)
cargo kani                                        # all 15 proofs
cargo kani --harness proof_grammar_compound_precedence  # single proof

# Run on synthetic pipeline data
cargo run -- data/pipeline_synthetic.csv

# Regenerate all synthetic datasets
python3 scripts/gen_synthetic_data.py

# Build PDF paper
bash scripts/build_paper.sh

# Generate all 20 figures from real data + compile booklet + zip
cargo run --release -- generate-figures

# Step-by-step figure pipeline:
#   1. Export DSFB grammar traces for all 3 real datasets
cargo run --release --example export_grammar_traces
#   2. Generate 20 PDF figures into a fresh timestamped output directory
DSFB_OUTPUT=/tmp/dsfb-oil-gas-example-output python3 scripts/gen_figures.py
#   3. Copy the booklet source and compile from that output directory
cp figures/all_figures.tex /tmp/dsfb-oil-gas-example-output/figures/
cd /tmp/dsfb-oil-gas-example-output/figures && pdflatex -interaction=nonstopmode all_figures.tex

# Open the bundled notebook (40 cells, real-data only + Kani formal verification)
jupyter notebook notebook/dsfb_oil_gas.ipynb
# … or run it headlessly (executes all cells, writes outputs in-place)
bash scripts/run_notebook.sh

# Export all artifacts
bash scripts/export_artifacts.sh
```

**Prerequisites:** Rust 1.70+, Python 3.8+, TeX Live 2022+.

---

## Mathematical Definition

For one scalar channel, the executable Rust implementation computes the DSFB
state in four deterministic stages:

1. Residual:
   `r_k = observed_k - expected_k`
2. Drift:
   `δ_k^(w) = (1 / m_k) Σ_{j=max(0, k-w+1)}^k r_j`, where `m_k = min(w, k+1)`
3. Slew:
   `σ_k = (r_k - r_{k-1}) / Δt_k`, with `σ_0 = 0`
4. Envelope normalisation:
   each coordinate is mapped affinely into `[-1, 1]`

The normalized coordinates are

```text
r̃_k = (r_k     - (r_min     + r_max    ) / 2) / ((r_max     - r_min    ) / 2)
δ̃_k = (δ_k^(w) - (delta_min + delta_max) / 2) / ((delta_max - delta_min) / 2)
σ̃_k = (σ_k     - (sigma_min + sigma_max) / 2) / ((sigma_max - sigma_min) / 2)
```

and the grammar precedence implemented in `src/grammar.rs` is

```text
Compound > EnvViolation > SlewSpike > DriftAccum > BoundaryGrazing > Recovery > Nominal
```

Two implementation details matter for faithful reproduction:

- The Rust code uses the **partial-window mean** during drift warm-up (`m_k = min(w, k+1)`). This is slightly more precise than the paper's simplified warm-start prose in Section II.
- `Δt_k` falls back to `1.0` on the first sample or whenever timestamps do not strictly increase. This prevents division-by-zero and keeps the pipeline deterministic, but it also means `σ_k` is only a physical rate when the caller supplies a physically meaningful monotone axis.

In practice this means:

- For time-series channels, `timestamp` should be seconds or another true time base.
- For depth-indexed drilling data, `timestamp` can be depth or step index, but then `σ_k` is a discrete gradient along that axis rather than a time derivative.
- For snapshot corpora such as ESPset, `σ_k` is a sequence-order difference, not a machine-dynamics derivative.

The executable grammar also includes one extra sentinel behavior that matters operationally:

- Non-finite residuals (`NaN`, `+∞`, `-∞`) emit `SensorFault`.
- Those samples do **not** update the internal drift ring buffer, slew state, or previous timestamp, so one bad point does not poison subsequent arithmetic.
- `Recovery` is intentionally a **single-step transient**; after one emitted recovery token, the predecessor state is reset to `Nominal`.

---

## Minimal Rust API Usage

For an existing residual stream:

```rust
use dsfb_oil_gas::{
    aggregate_episodes, format_summary, summarise,
    AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier, ResidualSample,
};

fn main() {
    let env = AdmissibilityEnvelope::default_pipeline();
    let mut engine = DeterministicDsfb::with_window(
        env,
        GrammarClassifier::new(),
        10,
        "pipeline_flow_balance",
    );

    let samples = [
        ResidualSample::new(0.0, 101.2, 100.9, "pipeline_flow_balance"),
        ResidualSample::new(1.0, 101.6, 101.0, "pipeline_flow_balance"),
        ResidualSample::new(2.0, 106.8, 101.1, "pipeline_flow_balance"),
    ];

    for sample in &samples {
        engine.ingest_sample(sample);
    }

    let episodes = aggregate_episodes(engine.history());
    let summary = summarise("pipeline_flow_balance", engine.history(), &episodes);
    println!("{}", format_summary(&summary));
}
```

For the built-in domain frame types, call `engine.ingest(frame)` instead of
constructing `ResidualSample` manually.

API notes that are easy to miss:

- `DeterministicDsfb::with_window(...)` sets the channel label stored in `history()`.
- `ResidualSample.channel` is informational on input, but the emitted `AnnotatedStep.channel` comes from the engine instance.
- `DeterministicDsfb::events()` returns only non-`Nominal` annotated steps; `aggregate_episodes(engine.history())` is the episode-level view.
- `ReadOnlySlice<T>` and `process_read_only(...)` are the crate's explicit read-only integration helpers.

For real channels, the important distinction is between **illustrative defaults**
and **paper-reproduction defaults**:

- `default_pipeline()`, `default_drilling()`, `default_rotating()`, `default_subsea()` are generic research defaults for synthetic examples.
- `default_oilwell()`, `default_volve_drilling()`, and `default_esp_rotating()` reproduce the bundled paper calibrations for the included real datasets.
- None of those defaults should be treated as universal field thresholds. They are channel- and regime-specific choices.

---

## Reproducing The Executable Results

From the crate root (`crates/dsfb-oil-gas`), the main entrypoints are:

```bash

# Real-data metric summaries used in the paper discussion
cargo run --example metrics_3w
cargo run --example metrics_volve
cargo run --example metrics_esp

# Demo binary: synthetic pipeline CSV
cargo run -- data/pipeline_synthetic.csv

# Real-data integration tests
cargo test --test real_data_3w
cargo test --test real_data_volve
cargo test --test real_data_esp

# Export per-step grammar traces used by the figure pipeline
cargo run --release --example export_grammar_traces

# Generate the full figure set into a fresh timestamped output directory
cargo run --release -- generate-figures

# Compile the paper directly
cd paper
pdflatex -interaction=nonstopmode dsfb_oil_gas.tex
pdflatex -interaction=nonstopmode dsfb_oil_gas.tex
```

What those commands correspond to empirically:

- `metrics_3w` processes the bundled Petrobras 3W real subset on P-MON-CKP.
- `metrics_volve` processes the bundled Equinor Volve 15/9-F-15 TQA trace.
- `metrics_esp` processes the bundled RPDBCS ESPset broadband-RMS trace.
- `generate-figures` runs the shared Rust trace export, the Python figure builder, booklet compilation, and archive packaging, and creates a new `output-dsfb-oil-gas/dsfb-oil-gas-YYYY-MM-DD-HHMMSS/` directory on each run.

---

## Evidence Boundary And Academic Honesty

The crate, tests, and paper support a narrower set of claims than a marketing
summary would suggest. The distinctions below are intentional.

What is established by the code and tests:

- Deterministic replay for identical inputs and parameters
- No mutable access through the provided `ReadOnlySlice<T>` integration wrapper
- Stable grammar precedence, including `Compound` and single-step `Recovery`
- Finite outputs for finite inputs and explicit sentinel handling for non-finite inputs
- Reproducible episode aggregation and summary metrics for the bundled datasets

What is established by the bundled empirical runs:

- DSFB executes end-to-end on three open datasets included with this crate:
  Petrobras 3W (9,087 timesteps), Equinor Volve drilling (5,326 depth-steps),
  and RPDBCS ESPset (6,032 snapshots).
- For the bundled 3W CSV specifically, the raw subset contains 12 episode IDs
  and 9,300 rows, but `load_oilwell_csv()` filters out rows where both choke
  values are zero. The executable choke-channel path therefore processes 9,087
  usable steps across 11 choke-populated episodes. This distinction matters
  when comparing the code output to the paper narrative.
- The real-data metrics are computationally reproducible from the bundled code
  and data, but some paper bookkeeping is simplified relative to the current
  executable outputs. When README text, paper prose, and example output differ,
  the current example/test output in this repository version should be treated
  as the authoritative reference.
- The 3W and Volve datasets behave like continuous traces and yield high NCR
  values around 18.8 to 18.9 under the crate's encoding model.
- ESPset behaves differently because it is a concatenated snapshot corpus rather
  than one continuous historian stream; its NCR of 1.5 is structurally lower and
  should be interpreted as such.

What is **not** established here:

- No field trial, hardware-in-the-loop evaluation, or production deployment validation
- No anomaly-detection accuracy claim, no prognostics claim, and no root-cause diagnosis claim
- No real-data validation for the pipeline or subsea mappings bundled in the architecture
- No certification claim under IEC 61508, IEC 61511, IEC 62443, API 1164, API 1165, API RP 1173, PHMSA CRM, or API 17F
- No claim that the provided envelope defaults generalise across wells, pumps, rigs, or operating regimes

Additional scope notes that matter:

- The non-intrusive claim is an **API and architecture claim** for the provided crate interfaces. It is not a certification statement about arbitrary surrounding deployment code.
- The Kani harnesses are useful bounded proofs over the implemented invariants in `src/kani_proofs.rs`; they are not a proof of calibration correctness, field robustness, or system-level safety.
- `noise_compression_ratio()` is an **estimated encoding ratio** from the explicit 32-byte raw-sample vs 40-byte episode-record model in `src/report.rs`. It is a consistent internal comparison metric, not a universal storage benchmark.
- In the ESP study, the closeness between `EnvViolation` rate and true fault prevalence is reported as a numerical correspondence only. It is **not** presented by the crate or the paper as a detection-performance result.
- The current ESP executable output includes a nonzero `Compound` state. Any prose summary that omits it should be read as a simplification, not as the canonical token ledger for this crate revision.

---

## Four Domain Modules

| Domain | Frame struct | Channel | Expected source |
|--------|-------------|---------|-----------------|
| Pipeline | `PipelineFrame` | `pipeline_flow_balance` | RTTM mass balance |
| Drilling | `DrillingFrame` | `drilling_torque_kNm` | Torque-and-drag model |
| Rotating equipment | `RotatingFrame` | `rotating_head_m` | Pump/compressor curve |
| Subsea actuation | `SubseaFrame` | `subsea_actuation_pressure` | Valve actuation model |

> **Empirical scope note.** Real-data evaluation covers drilling (Volve), oil-well events (3W), and rotating equipment (ESPset only). Claims for the pipeline and subsea domains are architectural mappings; no real pipeline or subsea datasets are included.

---

## Data

**Synthetic datasets** (analytical signal models; used for per-domain grammar verification):
- `data/pipeline_synthetic.csv` — Darcy–Weisbach pipeline residuals
- `data/drilling_synthetic.csv` — stick-slip torque residuals
- `data/rotating_synthetic.csv` — ESP head residuals
- `data/subsea_synthetic.csv` — HPU actuation pressure residuals

**Real datasets** (executed on actual Rust implementation; results reported in paper §VIII.D–G):
- `data/oilwell_real.csv` — Petrobras 3W Dataset v2.0.0 (CC BY 4.0); 12 WELL-* instances; 9,087 timesteps; six fault categories
- `data/drilling_real.csv` — Equinor Volve well 15/9-F-15 WITSML logs (Equinor Volve Data Licence V1.0); 5,326 depth-steps; surface-torque TQA channel
- `data/rotating_real.csv` — RPDBCS ESPset (MIT License); 6,032 vibration snapshots; 11 ESP pump units; five fault classes

All synthetic CSVs are computer-generated from the domain physics equations in the paper.
All real CSVs are derived from the original datasets under their respective open licenses.
No real data was used during envelope calibration; no label information was provided to the grammar automaton.

---

## Paper

`paper/dsfb_oil_gas.tex` — Full monograph-style paper.
`paper/bibliography.bib` — BibTeX references.

Build: `bash scripts/build_paper.sh` → `paper/dsfb_oil_gas.pdf`

---

## Figures

20 production-quality figures generated from the three real-world datasets.
Figures 16 and 17 additionally include synthetic reference bars (grey) for
cross-dataset comparison; all other figures contain no synthetic or simulated data.

**Generate everything:** `cargo run --release -- generate-figures`\
Outputs: `output-dsfb-oil-gas/dsfb-oil-gas-YYYY-MM-DD-HHMMSS/figures/fig_*.pdf` (20 individual), `all_figures.pdf` (compiled booklet), `dsfb_figures.zip` (download archive)

| Figure | Dataset | Description |
|--------|---------|-------------|
| fig_01_3w_residual_annotated | Petrobras 3W | Full 9,087-step P-MON-CKP residual, token-coloured background |
| fig_02_3w_token_per_well | Petrobras 3W | Per-well token distribution (stacked horizontal bar) |
| fig_03_3w_phase_portrait | Petrobras 3W | Phase portrait (r̃, δ̃) coloured by token |
| fig_04_3w_episode_ecdf | Petrobras 3W | ECDF of episode lengths (log-x) |
| fig_05_3w_observed_expected | Petrobras 3W | Observed vs expected pressure by event class |
| fig_06_volve_tqa_annotated | Equinor Volve | TQA residual vs depth, grammar token shading |
| fig_07_volve_drift_slew | Equinor Volve | Two-panel δ and σ vs depth with envelope bounds |
| fig_08_volve_token_dist | Equinor Volve | Token distribution horizontal bar |
| fig_09_volve_episode_hist | Equinor Volve | Episode length histogram (log–log) |
| fig_10_volve_phase_portrait | Equinor Volve | Phase portrait (r̃, δ̃) coloured by token |
| fig_11_esp_per_unit_envviol | RPDBCS ESP | Per-unit true fault rate vs EnvViolation rate (post-hoc) |
| fig_12_esp_rms_by_class | RPDBCS ESP | Broadband RMS by fault label (notched box plot) |
| fig_13_esp_token_dist | RPDBCS ESP | Token distribution horizontal bar |
| fig_14_esp_residual_units | RPDBCS ESP | Residual + token trace for ESP units 0, 1, 4 |
| fig_15_esp_phase_portrait | RPDBCS ESP | Phase portrait coloured by fault label (post-hoc) |
| fig_16_cross_ncr_bar | All 3 real | Cross-dataset NCR bar chart |
| fig_17_cross_edr_bar | All 3 real | Cross-dataset EDR bar chart |
| fig_18_cross_token_heatmap | All 3 real | Dataset × token heatmap (%) |
| fig_19_cross_episode_violin | All 3 real | Violin plot episode lengths per dataset |
| fig_20_cross_envelope_utilisation | All 3 real | Fraction steps with ‖·‖ > 1 per dataset |

**Figures integrated into paper** (panel-selected, 6 primary):
fig_01, fig_06, fig_07, fig_11, fig_16, fig_18

**Intermediate trace data:** `figures/trace_data/real_*_trace.csv` — per-step grammar annotation CSVs
produced by `cargo run --release --example export_grammar_traces`.

---

## Limitations (Summary)

1. Sensor-quality dependence — DSFB cannot detect sensor drift or calibration errors
2. Envelope calibration burden — all six parameters require domain-expert tuning per channel
3. No predictive capability — describes what happened; does not forecast
4. Window-size sensitivity — window w must match expected drift timescale
5. Nyquist-limited — sub-Nyquist transients are not observable
6. Single-channel — no multi-sensor correlation analysis
7. Human-readable ≠ physically correct — poor expected-value model → misleading tokens
8. No certified safety function — not IEC 61511/61508 qualified; no SIL rating
9. No certification claim of any kind — standards traceability is not a certification claim
10. Modes 3–4 (edge/live shadow) not yet implemented — TRL 3 only

---

## Standards Traceability Matrix

DSFB is designed for deployment alongside systems governed by the following standards. This matrix states what DSFB does in relation to each standard, and what it explicitly does **not** claim.

> **No certification is claimed. Alignment is structural and documentable; it is not a substitute for certified compliance.**

| Standard | Governs | DSFB Does | DSFB Does NOT Claim |
|---|---|---|---|
| **IEC 61511** | Functional safety — process-sector SIS | Read-only observer; no write to SIS logic, actuators, or alarm limits; removal leaves SIS unchanged | SIS component; safety instrumented function; SIL 1–4 rating; 61511-qualified software |
| **IEC 61508** | Functional safety — E/E/PE systems | Observer-only; DSFB failure produces no upstream change; no common-cause coupling to safety path | Certified under IEC 61508; SIL rated; part of any safety function |
| **IEC 62443** | Industrial cybersecurity (OT/ICS) | Least-privilege deployment; no inbound network port; no internet dependency; pinned `Cargo.lock`; `cargo audit` scanning | Certified IEC 62443 product; SL-1 or higher; verified secure component per 62443-4-2 |
| **WITSML (Energistics)** | Drilling data interoperability | `DrillingFrame` channel labels align with WITSML mnemonics (TORQUE, WOB, RPMA); structural CSV compatibility | WITSML server implementation; certified Energistics interoperability |
| **API 1164** | Pipeline SCADA cybersecurity | Read-only historian access; annotation namespace separate from control namespace; no new inbound ports | Certified API 1164 product |
| **API 1165** | Pipeline SCADA display / HMI | Tokens are supplemental text/log data; DSFB generates no alarm graphics or control display elements | SCADA display component; alarm management system; HMI replacement |
| **API RP 1173** | Pipeline Safety Management Systems | Deterministic event log suitable as supplemental integrity engineering record | PSMS component; certified PSMS record system |
| **PHMSA 49 CFR 195 (CRM)** | Control Room Management (pipeline) | Event log provides structured inter-alarm narrative; does not issue controller alerts; does not modify display content | CRM-certified monitoring tool; abnormal operations management system |
| **API 17F** | Subsea production control systems | Observer-only on HPU pressure historian tag; no interface to EIM, valve controller, or MCS | Subsea control system component; 17F certified; subsea safety function |

---

## Claims Boundary Table

Every **DOES NOT** is a hard boundary, not a limitation pending future work.

| Capability | Status | Justification |
|---|---|---|
| Safety system (SIS component) | **DOES NOT** | No SIL rating; no 61508/61511 qualification; no actuator interface; read-only by construction |
| Control system | **DOES NOT** | No write path to control variable, setpoint, PLC register, or DCS tag |
| Predictive maintenance system | **DOES NOT** | Strictly retrospective; no RUL estimation; no failure prediction |
| Leak detection system | **DOES NOT** | Not an API RP 1175 certified LD system; performs no mass-balance computation |
| Certified cybersecurity product | **DOES NOT** | Not certified under IEC 62443 or API 1164; alignment with secure development practices documented only |
| WITSML server implementation | **DOES NOT** | Channel mapping alignment only; does not implement WITSML query/response/transport protocol |
| SCADA / HMI replacement | **DOES NOT** | Token stream is supplemental log data; not a display component, not an alarm source |
| Root-cause analysis engine | **DOES NOT** | Annotates residual structure; does not infer physical causation; does not assign fault to component |
| Field-validated system | **DOES NOT** | No operational SCADA/historian connection; no HIL rig; no field pilot; three real open-license datasets validated computationally at TRL 3 (Petrobras 3W, Equinor Volve, RPDBCS ESPset) — this is not field validation |
| Read-only analytical observer | **DOES** | No upstream modification; immutable input; test-verified (A1–A4); removal restores baseline |
| Deterministic replay guarantee | **DOES** | Same input → same output; verified by `deterministic_replay_identical` test |
| Episode compression | **DOES** | NCR > 6:1 across all four domains; ECC 8.7–16.8× |
| Inter-alarm event structuring | **DOES** | Grammar token sequence annotates residuals between existing alarm events |

---

## Non-Intrusive Assurance Package

Four demonstrable assurance properties, each tested:

| Property | Mechanism | Test |
|---|---|---|
| **A1 — Immutable input** | `ReadOnlySlice<T>` type wrapper; no `&mut`reference to upstream data | `dsfb_does_not_modify_input_samples`, `read_only_slice_values_unchanged` |
| **A2 — No control output** | No write path in codebase; annotation written to dedicated output only | Architecture (code review); `process_read_only_cannot_mutate_source` |
| **A3 — Deterministic replay** | No internal random state; no global mutable state; no clock dependency | `deterministic_replay_identical`, `replay_is_deterministic` |
| **A4 — Removable** | No persistent side effect on upstream; no schema change; no PLC modification | Architecture (deployment guide) |

---

## Deployment Modes

| Mode | Input | Output | System Interaction | Status |
|---|---|---|---|---|
| 1. Offline analysis | Historical CSV / historian export | Episode CSV + token sequence | Post-hoc; no live system contact | **Implemented** |
| 2. Historian replay | Live historian read API (read-only) | JSONL event log → annotation namespace | Read + annotation write; control namespace untouched | **Implemented** |
| 3. Edge observer | Real-time SCADA tag stream (read-only) | Streaming token buffer on edge device | Read-only; OT DMZ deployment | **Future — not implemented** |
| 4. Live shadow | Real-time RTTM / SCADA residual feed | Shadowed episode log + ECC metrics | Read-only; requires HWIL validation | **Future — not implemented** |

---

## Operator Environment Awareness

**Alarm fatigue:** DSFB does not generate alarms. The grammar token stream is supplemental information only. Any downstream integration that converts tokens into operator alerts must be designed and validated separately with explicit attention to nuisance alarm rate.

**Display constraints (API 1165):** Token data rendered on SCADA displays must be clearly labeled as advisory. DSFB does not provide colour-coded alarm states or control-interlock display elements. HMI designers are responsible for API 1165 compliance.

**Control authority separation:** No token at any precedence level implies a required operator action. Control authority — SCADA, DCS, PLC, SIS — is unaffected by DSFB operation or failure.

---

## Secure Development and Cybersecurity Alignment

> **This is NOT a certified IEC 62443 product.** This section demonstrates alignment with secure development expectations at TRL 3.

| Practice | Implementation |
|---|---|
| **Dependency pinning** | `Cargo.lock` committed; all deps pinned to exact versions |
| **Minimal dependency surface** | `csv 1.3`, `serde 1.0`, `thiserror 1.0` only; no networking, no crypto, no `unsafe` |
| **Vulnerability scanning** | `cargo audit` against RustSec Advisory Database; run before any deployment |
| **SBOM generation** | `cargo cyclonedx` or `cargo sbom` produces CycloneDX / SPDX SBOM |
| **Least privilege** | Read-only historian access + annotation write only; no root / admin required; no listening port |
| **Network boundary** | Runs inside OT DMZ; no internet access required at runtime |
| **Audit trail** | Deterministic JSONL output is an implicit audit trail; reproduces exactly from archived input |
| **Release signing** | Not currently implemented (TRL 3); apply binary signing per org security policy before production use |

---

## TRL Table

| Level | Description | Status |
|---|---|---|
| TRL 1 | Basic principles observed | ✓ |
| TRL 2 | Technology concept formulated | ✓ |
| TRL 3 | Proof-of-concept on synthetic benchmarks (all 4 domains) + real-dataset computational validation (Petrobras 3W, Equinor Volve, RPDBCS ESP) | ✓ **Current** |
| TRL 4 | Technology validated in lab (HIL / emulated SCADA) | Pending — Phase II |
| TRL 5 | Technology validated in relevant environment (field pilot) | Future |
| TRL 6–9 | Demonstration / production | Not applicable at this stage |

---

## Phase I SBIR Deliverables

| Deliverable | Validation Performed | Standards Relevance | NOT Delivered in Phase I |
|---|---|---|---|
| Validated Rust crate (52 tests passing) | Unit + integration test suite | IEC 61508 (observer layer, non-SIL) | SIL assessment; safety case |
| Synthetic benchmarks (4 domains) | Deterministic replay verification | Internal validation | Sole basis for domain claims |
| Real 3W sensor run (9,087 steps, 12 instances, 6 fault types, CC BY 4.0) | Rust crate executed on real residuals; NCR = 18.8 | Empirical feasibility | Field validation; multi-channel; certified LD |
| Real Volve drilling run (5,326 depth-steps, TQA, Equinor Data Licence) | Rust crate executed on real residuals; NCR = 18.9 | Empirical feasibility | Cross-well generalisation; formation label correlation |
| Real ESP rotating run (6,032 snapshots, RPDBCS, MIT License) | Rust crate executed on real residuals; EnvViolation 20.7% vs 20.4% fault rate | Empirical feasibility | Continuous streaming NCR; drive-train coupling |
| Non-intrusive assurance package (A1–A4) | Test-verified properties | IEC 61511, IEC 61508 alignment | Formal safety case; FTA |
| Standards Traceability Matrix (9 standards) | Expert review artifact | IEC 61511/61508/62443, API, WITSML | Certified compliance |
| Claims Boundary Table | Expert review artifact | All applicable standards | Legal certification |
| Deployment modes definition | Architecture documentation | PHMSA CRM, API 1165 | Deployed edge system |
| Secure development alignment | `cargo audit` + `Cargo.lock` documentation | IEC 62443 alignment | Certified IEC 62443 product |
| IEEE-format paper | Peer review ready | Technology documentation | Regulatory filing |

---

## License

Apache-2.0. Commercial deployment requires separate license from Invariant Forge LLC.

## Citation

de Beer, R. (2026). *DSFB: Structural Residual Semiotics Engine\\
for Upstream and Midstream Oil and Gas Systems* (v1.0). Zenodo.
<https://doi.org/10.5281/zenodo.19549262>

## Citation

If you use this work, please cite:

```
@software{dsfb_oil_gas_2026,
  author = {de Beer, Riaan},
  title = {DSFB: Structural Residual Semiotics Engine
for Upstream and Midstream Oil and Gas Systems},
  year = {2026},
  doi = {10.5281/zenodo.19549262},
  publisher = {Zenodo}
}
```

## Prior Art

This work constitutes prior art under 35 U.S.C. § 102, timestamped via
Zenodo DOI <https://doi.org/10.5281/zenodo.19549262> and crates.io `dsfb-oil-gas`
publication.

## IP Notice

The theoretical framework, formal constructions, and supervisory methods
described herein constitute proprietary Background IP of Invariant Forge LLC
(Delaware LLC No.\ 10529072), with prior art established by this publication
and earlier Zenodo DOI publications by the same author.  Commercial deployment
requires a separate written license.  Reference implementations are released
under Apache~2.0.  Licensing:
licensing@invariantforge.net
