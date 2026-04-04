# dsfb-semiconductor

[![Crates.io](https://img.shields.io/crates/v/dsfb-semiconductor.svg)](https://crates.io/crates/dsfb-semiconductor)
[![Docs.rs](https://docs.rs/dsfb-semiconductor/badge.svg)](https://docs.rs/dsfb-semiconductor)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb)

`dsfb-semiconductor` is a **deterministic, non-intrusive structural semiotics
engine** for semiconductor process control — a read-only augmentation layer that
operates over typed residual streams, producing advisory episode classifications
without touching upstream SPC, APC, FDC, or EWMA/CUSUM controllers.

The crate is the empirical software companion for the DSFB semiconductor paper
(see [Citation](#citation) below). It instantiates the paper's bounded claim:
DSFB compresses raw structural alarms into inspectable, precision-governed
episodes while preserving full failure coverage.

On the SECOM benchmark with default parameters:

| Metric | Raw | DSFB | Change |
|--------|-----|------|--------|
| Review episodes | 28,607 | 71 | -99.75 % |
| Episode precision | 0.36 % | 80.3 % | 220.8x |
| Investigation-worthy decisions | 10,554 | 3,854 | -63.5 % |
| Failure-labeled runs covered | 104 | 104 | 0 (full) |

---

## Table of Contents

- [Design principles](#design-principles)
- [Feature flags](#feature-flags)
- [Mathematical foundation](#mathematical-foundation)
  - [Residual construction](#residual-construction)
  - [Sign computation -- drift and slew](#sign-computation--drift-and-slew)
  - [Grammar -- admissibility finite-state machine](#grammar--admissibility-finite-state-machine)
  - [Syntax -- motif classification](#syntax--motif-classification)
  - [Semantics -- heuristics-bank lookup](#semantics--heuristics-bank-lookup)
  - [Policy -- decision ranking](#policy--decision-ranking)
  - [DSA -- deterministic structural accumulation](#dsa--deterministic-structural-accumulation)
  - [Baselines included for comparison](#baselines-included-for-comparison)
  - [Process context -- recipe-step admissibility and maintenance hysteresis](#process-context--recipe-step-admissibility-and-maintenance-hysteresis)
  - [Multivariate observer](#multivariate-observer)
- [Code architecture](#code-architecture)
- [Library usage](#library-usage)
  - [Minimal no_std observer](#minimal-no_std-observer-zero-copy-no-heap)
  - [Full pipeline -- fab sidecar pattern](#full-pipeline--fab-sidecar-pattern)
  - [Process-context gating](#process-context-gating)
  - [Signature file lock](#signature-file-lock)
- [Data preparation](#data-preparation)
- [CLI quickstart](#cli-quickstart)
- [Supported datasets](#supported-datasets)
- [Output artifacts](#output-artifacts)
- [no_std kernel](#no_std-kernel)
- [Reproducibility discipline](#reproducibility-discipline)
- [Caveats and non-claims](#caveats-and-non-claims)
- [Citation](#citation)

---

## Design principles

DSFB is a **supervisory, read-only** system. There is no write path.

| Guarantee | Enforcement |
|-----------|-------------|
| No mutation of upstream data | Observer API accepts only `&[f64]` (shared reference) |
| No control-path influence | No write path into any upstream structure |
| Deterministic under identical inputs | Pure function composition over fixed parameters |
| Removable without system impact | Advisory outputs only; zero coupling |
| No side effects | Kernel `observe()` is a pure function |

If DSFB stops running, the upstream plant is entirely unaffected.
SPC, EWMA, threshold logic, APC, and controller behaviour remain authoritative.

---

## Feature flags

| Feature | Default | Effect |
|---------|---------|--------|
| `std` | **yes** | Enables CLI, I/O, dataset adapters, plotting, calibration, and all benchmark pipeline modules. |
| *(none, `--no-default-features`)* | -- | Kernel-only build: `sign`, `grammar`, `syntax`, `semantics`, `policy`, `process_context`, `units`. Suitable for bare-metal, RTOS, and FPGA deployments. Requires only `alloc`. |

---

## Mathematical foundation

### Residual construction

Given a feature vector x(k) in R^p at sample index k, the nominal reference
model is computed from the first N_h healthy-window observations:

```
mu_i   = (1/N_h) * sum x_i(k)                      (healthy window mean)
sigma_i = sqrt((1/N_h) * sum (x_i(k) - mu_i)^2)    (healthy window std)
rho_i  = sigma_env * sigma_i                         (envelope radius, default sigma_env = 3)
r_i(k) = |x_i(k) - mu_i|                            (per-feature residual norm)
```

Missing values (NaN) are tracked by an imputation mask and replaced with mu_i
before norm computation, ensuring the pipeline never propagates non-finite values
downstream.

All thresholds are computed from the healthy window at run time and saved verbatim
to `parameter_manifest.json` for every run. There are no pre-fit magic constants.

### Sign computation -- drift and slew

The **drift** of feature i at index k is the finite-difference slope of the
residual norm over a rolling window of width W_d:

```
drift_i(k) = (r_i(k) - r_i(k - W_d)) / W_d     (default W_d = 5)
```

Drift captures the directional rate of change: positive drift means the residual
is growing; negative drift means it is recovering. Imputed samples zero-out the
drift for that window.

The **slew** is the first difference of drift (acceleration of residual change):

```
slew_i(k) = drift_i(k) - drift_i(k-1)
```

Both drift and slew thresholds are set at 3 sigma of the healthy-window
distribution of the respective sign:

```
delta_i = 3 * std(drift_i during healthy window)
zeta_i  = 3 * std(slew_i  during healthy window)
```

The sign triple (r_i, drift_i, slew_i) forms the input to the grammar layer.
This representation encodes where the residual is, how fast it is moving, and how
that speed is changing -- without any probabilistic model.

### Grammar -- admissibility finite-state machine

The grammar FSM maps the residual norm and drift onto one of three states per
feature per sample:

| State | Condition |
|-------|-----------|
| `Admissible` | r_i(k) <= rho_i and no sustained outward drift |
| `Boundary` | r_i(k) in (0.5*rho_i, rho_i] with positive drift, or recurrent boundary grazing |
| `Violation` | r_i(k) > rho_i |

**Recurrent grazing** fires `Boundary` when at least `grazing_min_hits` out of
the last `grazing_window` samples exceeded 0.5*rho_i, even if no individual
sample has crossed rho_i. This captures slow creep that would be invisible to a
simple threshold.

**Hysteretic confirmation** (controlled by `state_confirmation_steps`) requires
a new state to persist for at least C consecutive steps before the FSM
transitions. This prevents flickering between states on marginal samples.

**Persistent state masks** are derived after hysteresis: a feature is in
`persistent_boundary` or `persistent_violation` if the confirmed grammar state
has held for at least `persistent_state_steps` consecutive indices.

The grammar evaluates each feature independently. The resulting `GrammarSet`
carries a full `FeatureGrammarTrace` per feature, including raw states, confirmed
states, suppression flags, and both persistent masks.

In the minimal `observe()` kernel the grammar uses only squared norms to avoid
`sqrt` and remain `no_std`-compatible:
r^2 > rho^2 -> Violation, r^2 > (0.5*rho)^2 and drift > 0 -> Boundary, else Admissible.

Grammar reasons:
- `SustainedOutwardDrift`    -- drift > delta_i for C consecutive steps
- `AbruptSlewViolation`      -- slew > zeta_i in a single step
- `RecurrentBoundaryGrazing` -- grazing density gate fires
- `EnvelopeViolation`        -- r_i > rho_i directly

### Syntax -- motif classification

The syntax layer classifies the time-ordered sign sequence for each feature into
one of eight named motifs:

| Motif | Description |
|-------|-------------|
| `slow_drift_precursor` | Sustained positive drift below the violation envelope -- the classic pre-failure creep pattern |
| `boundary_grazing` | Repeated boundary touches without full violation -- persistent instability near the edge |
| `transient_excursion` | A brief violation that decays quickly back to admissible |
| `persistent_instability` | Extended violation or recurrent boundary-grazing over many consecutive samples |
| `burst_instability` | High-slew entry into violation -- abrupt excursion rather than gradual drift |
| `recovery_pattern` | Confirmed return to admissible after prior non-admissible state |
| `noise_like` | Small, unstructured residual fluctuation; no directional coherence |
| `null` | Insufficient data or all-imputed window |

Classification uses envelope statistics (inter-quartile range, median, peak norm)
computed from the sign sequence, combined with directional drift tests. Motif
boundaries are run-length encoded so each feature produces a compact `Vec<Motif>`
timeline, not a flat label-per-sample array.

The motif layer is what separates DSFB from a simple threshold: a threshold fires
whenever r > rho; the motif layer tracks *what kind* of structural event is
happening.

### Semantics -- heuristics-bank lookup

The semantics layer matches grammar-conditional motifs against a deterministic
heuristics bank. Each bank entry has the form:

```text
heuristic_id      -- unique identifier
grammar_condition -- required grammar state (e.g., "Boundary", "Violation")
motif_type        -- required motif (e.g., "slow_drift_precursor")
action            -- advisory output: "Silent" | "Watch" | "Review" | "Escalate"
severity_tag      -- informational: "low" | "medium" | "high"
action_note       -- human-readable rationale
limitations       -- explicit scope limit of this heuristic
```

A `SemanticMatch` is emitted only when both the grammar condition **and** the
motif type match the current feature state. This is the key gate that separates
structural patterns from random noise: a fragment that passes the grammar check
but does not match any motif stays `Silent`.

Default heuristics for key patterns:

| Pattern | Grammar | Action |
|---------|---------|--------|
| `slow_drift_precursor` | Boundary | Review |
| `boundary_grazing` (recurrent) | Boundary | Watch |
| `transient_excursion` (single) | Violation | Silent |
| `persistent_instability` | Violation | Escalate |
| `burst_instability` | Violation | Review |
| `recovery_pattern` | Admissible | Silent |

The heuristics bank is operator-overridable via `DsfbSignatureFile`, which locks
the entire configuration to a JSON file that can be version-controlled and
diff-audited across tool generations.

### Policy -- decision ranking

The policy layer merges grammar-level fallback signals with semantic-match actions
into a single ordered decision per timestamp, using strict rank promotion:

```
rank(Escalate) = 3
rank(Review)   = 2
rank(Watch)    = 1
rank(Silent)   = 0
```

For each timestamp k:
1. Grammar fallback: Admissible -> Silent, Boundary/Violation -> Watch
2. Semantic matches: promote timestamp rank to semantic.action if higher
3. The maximum rank across all features at timestamp k is the run-level output

This means a single Escalate-grade semantic match on any feature raises the
run-level decision. Conversely, a run with no semantic matches and only admissible
grammar is always Silent.

### DSA -- deterministic structural accumulation

DSA (Deterministic Structural Accumulation) is a per-feature score that aggregates
six structural inputs, each derived purely from the sign-level data:

| Input | Symbol | Description |
|-------|--------|-------------|
| Rolling boundary density | d_B(k) | Fraction of last W samples in Boundary |
| Drift persistence | d_P(k) | Fraction of last W samples with drift > delta_i |
| Slew density | d_S(k) | Fraction of last W samples with slew > zeta_i |
| EWMA occupancy | d_E(k) | Normalized EWMA residual norm relative to envelope |
| Motif recurrence | d_M(k) | Fraction of last W motif samples that are non-null |
| Directional consistency | d_C(k) | 1 if drift direction is consistently outward, else 0 |

The DSA score is a fixed linear combination:

```
DSA_i(k) = w_B*d_B + w_P*d_P + w_S*d_S + w_E*d_E + w_M*d_M + w_C*d_C
```

Default weights: w_B=1.0, w_P=1.0, w_S=0.5, w_E=0.5, w_M=0.5, w_C=0.5.

A **feature-level DSA alert** fires when:

```
DSA_i(k) >= tau    for at least K consecutive runs
```

The parameters W (window), K (persistence), tau (threshold), and m (corroboration
count) are the four calibration knobs. They are bounded and saved to
`dsa_parameter_manifest.json`.

A **run-level DSA alert** fires when at least m features are simultaneously in
Review or Escalate:

```
|{i : policy_i(k) in {Review, Escalate}}| >= m
```

The paper-selected configuration on SECOM is W=5, K=2, tau=2.0, m=2. The bounded
calibration grid covers W in {5,10,15}, K in {2,3,4}, tau in {2.0,2.5,3.0},
m in {1,2,3,5}, and all results are saved.

### Baselines included for comparison

| Baseline | Acronym | Formula |
|----------|---------|---------|
| Residual-magnitude threshold | THR | r_i(k) > rho_i |
| Univariate EWMA residual norm | EWMA | z_i(k) = alpha*r_i(k) + (1-alpha)*z_i(k-1); alarm at z_i > mu_z + 3*sigma_z |
| Positive CUSUM | CUSUM | C+(k) = max(0, C+(k-1) + r_i(k) - mu_r - kappa); alarm at C+ > h (kappa=0.5*sigma_r, h=5*sigma_r) |
| Run-energy scalar | RES | E(k) = (1/p)*sum r_i^2(k); alarm threshold from healthy E distribution |
| PCA T2/SPE multivariate FDC | PCA | Healthy-window PCA fit; Hotelling T2 statistic and SPE (Q-statistic) both at 3 sigma |

All five baselines are computed from the same healthy window used by the DSFB
grammar, and their results are saved alongside DSFB output for direct comparison.

### Process context -- recipe-step admissibility and maintenance hysteresis

Industrial processes are not stationary. The `process_context` module encodes two
kinds of domain knowledge that are invisible to purely data-driven methods:

**Recipe-step admissibility (LUT):** The envelope radius rho_i is scaled by a
step-specific multiplier before the grammar evaluation:

| Recipe step | LUT multiplier | Rationale |
|-------------|---------------|-----------|
| `GasStabilize` | 1.5x | Transient overshoots are physically expected during MFC ramp |
| `MainEtch` | 1.0x | Yield-critical window; full-sensitivity grammar |
| `Deposition` | 1.2x | Moderate tolerance |
| `OverEtch` | 1.1x | Slightly relaxed |
| `Seasoning` | 2.0x | Post-maintenance conditioning; widest tolerance |
| `Other` | 1.0x | Baseline |

**Maintenance hysteresis (Warm Reset):** When the tool signals `ChamberClean`,
the accumulated grammar state is cleared and a configurable guard window
(`post_clean_guard_runs`) suppresses new alarms for the first N_g runs.
This prevents spurious escalations during the seasoning period.

### Multivariate observer

The `multivariate_observer` module (std-only) implements a Hotelling T2/SPE
observer over all analyzable features simultaneously. A PCA model is fit on the
healthy-window residual matrix, retaining enough principal components to explain
`pca_variance_explained` (default 95%) of healthy variance.

At each sample k:

```
T2(k) = z(k)^T * Lambda^-1 * z(k)    (Hotelling statistic in PCA subspace)
Q(k)  = ||r(k) - r_hat(k)||^2         (SPE / Q-statistic)
```

where z(k) are the scores in the retained PCA subspace, Lambda the diagonal
eigenvalue matrix, and r_hat(k) the PCA reconstruction. Alarms fire at
T2 > tau_T2 or Q > tau_Q.

---

## Code architecture

```
src/
|-- lib.rs                   # Crate root; observe() minimal kernel API
|-- config.rs                # PipelineConfig -- all tunable parameters with validated defaults
|-- units.rs                 # Type-safe physical-quantity newtypes (f64 wrappers)
|-- sign/mod.rs              # FeatureSignPoint -- structured sign at a single timestamp
|-- signs.rs                 # compute_drift(), compute_slew() -- sign computation
|-- nominal.rs               # NominalModel -- healthy-window mean/sigma/rho per feature
|-- residual.rs              # ResidualSet -- r(k) = |x(k) - mu| per feature
|-- grammar.rs               # GrammarSet -- per-feature Admissible/Boundary/Violation FSM
|-- grammar/layer.rs         # Streaming six-state grammar for sign-stream integration
|-- syntax/mod.rs            # Motif classification -- 8 named motif types
|-- semantics/mod.rs         # SemanticMatch -- grammar-conditional heuristics-bank lookup
|-- policy/mod.rs            # PolicyDecision -- rank-promoted decision per timestamp
|-- process_context.rs       # RecipeStep LUT + MaintenanceHysteresis warm reset
|
|-- baselines.rs             # EWMA, CUSUM, PCA -- comparison baselines
|-- calibration.rs           # Deterministic parameter-grid calibration
|-- cohort.rs                # Feature cohort selection and delta-target assessment
|-- dataset/                 # SECOM and PHM 2018 dataset adapters
|-- failure_driven.rs        # Failure-labeled run diagnostics
|-- heuristics.rs            # Heuristics-bank construction and policy overrides
|-- input/                   # ResidualStream and AlarmStream sorted input types
|-- interface/mod.rs         # FabDataSource trait -- non-intrusive integration surface
|-- metrics.rs               # Episode precision, lead-time, density metrics
|-- missingness.rs           # Missing-value tracker and suppression rules
|-- multivariate_observer.rs # PCA T2/SPE observer
|-- non_intrusive.rs         # Architecture-spec artifact generation
|-- output_paths.rs          # Timestamped, non-reusing output directory paths
|-- pipeline.rs              # Full SECOM / PHM benchmark pipeline entry points
|-- plots.rs                 # PNG figure generation (grammar timeline, DRSC, DSA)
|-- precursor.rs             # DsaConfig + DSA scoring, persistence, cohort grid
|-- preprocessing.rs         # Dataset preparation, healthy-window split
|-- report.rs                # Markdown / LaTeX / PDF engineering report generation
|-- secom_addendum.rs        # SECOM-specific delta targets, rating-delta analysis
|-- semiotics.rs             # Grouped semiotics scaffold -- multi-feature sign scaffolds
|-- signature.rs             # DsfbSignatureFile -- version-controlled config lock
|-- traceability.rs          # Full chain: Residual -> Sign -> Motif -> Grammar -> Semantic -> Policy
|-- unified_value_figure.rs  # Unified SECOM + PHM paper figure
`-- cli.rs                   # clap-based CLI for all benchmark subcommands
```

---

## Library usage

### Minimal no_std observer (zero-copy, no heap)

```rust
use dsfb_semiconductor::observe;

let residuals: &[f64] = &[0.1, 0.2, 0.5, 1.2, 2.1, 0.3, 0.1];
let episodes = observe(residuals);

for e in &episodes {
    // Advisory only -- no write-back, no upstream coupling
    println!(
        "i={} |r|2={:.3} drift={:.3} grammar={} decision={}",
        e.index, e.residual_norm_sq, e.drift, e.grammar, e.decision
    );
}
// grammar  in { "Admissible", "Boundary", "Violation" }
// decision in { "Silent", "Review", "Escalate" }
```

`observe()` is a pure function. Identical inputs always produce identical outputs.
NaN/inf samples are automatically imputed as admissible.

### Full pipeline -- fab sidecar pattern

```rust
use dsfb_semiconductor::{
    config::PipelineConfig,
    interface::FabDataSource,
    input::residual_stream::ResidualStream,
};

// Build an advisory observer -- read-only, no write-back to upstream
let config = PipelineConfig::default();
let mut stream = ResidualStream::new();

// Push residual observations from your existing FDC or SPC system.
// The stream holds only &[f64] -- no mutation of upstream buffers.
for (feature_id, residual_value) in your_fdc_residuals() {
    stream.push(feature_id, residual_value, current_timestamp);
}

// Read the sorted, deterministic stream (order guaranteed)
let sorted = stream.sorted();
// All outputs are advisory; there is no write path.
```

For integration with an existing fab data system, implement the `FabDataSource` trait:

```rust
use dsfb_semiconductor::interface::FabDataSource;

struct MyFdcAdapter { /* your fields */ }

impl FabDataSource for MyFdcAdapter {
    fn feature_ids(&self) -> Vec<String> { /* ... */ }
    fn residual_at(&self, feature_id: &str, index: usize) -> Option<f64> { /* ... */ }
    fn sample_count(&self) -> usize { /* ... */ }
}
```

The `FabDataSource` trait is intentionally read-only: there is no `write` or `set`
method. This makes the non-intrusion guarantee structurally enforced by the type system.

### Process-context gating

```rust
use dsfb_semiconductor::process_context::{
    RecipeStep, ToolState, MaintenanceHysteresis, AdmissibilityLut,
};

// Recipe-step LUT scales the grammar envelope per step
let lut = AdmissibilityLut::default();
let rho_main_etch = lut.scale(RecipeStep::MainEtch) * base_rho;     // 1.0x
let rho_gas_stab  = lut.scale(RecipeStep::GasStabilize) * base_rho; // 1.5x

// Maintenance hysteresis: suppress grammar for 5 runs after chamber clean
let mut hysteresis = MaintenanceHysteresis::new(5);
let grammar_active = hysteresis.update(ToolState::ChamberClean);
// grammar_active == false for next 5 calls to update()
```

### Signature file lock

The `DsfbSignatureFile` type provides a JSON-serializable configuration lock that
pins algorithm version, all pipeline parameters, and the full heuristics bank to a
file. Any parameter change is explicit, diff-auditable, and tied to a version
identifier:

```rust
use dsfb_semiconductor::signature::DsfbSignatureFile;
use std::path::Path;

// Create a signature from the current config + heuristics bank
let sig = DsfbSignatureFile::from_config(&config, &heuristics_bank, "v1.0.0-secom");
sig.write(Path::new("dsfb_signature.json"))?;

// Load and verify -- fails if any field changed
let loaded = DsfbSignatureFile::load(Path::new("dsfb_signature.json"))?;
assert_eq!(loaded.schema_version, "1");
```

---

## Data preparation

The crate ships **no dataset files**. All benchmark data is either downloaded at
runtime or must be placed manually before running any pipeline subcommand.
Skipping this step causes an immediate `DatasetMissing` error.

### Step 1 — Fetch SECOM (automated, ~4 MB)

SECOM is downloaded automatically from the UCI ML Repository the first time you
run `fetch-secom`. The archive is cached so subsequent runs are instant.

```bash
# Run once, from the crate directory (or wherever you want the data/ folder)
cargo run --release -- fetch-secom
```

This creates:

```
data/raw/secom/
  secom.data           # 1567 rows x 590 features, space-separated, NaN as -1 or missing
  secom_labels.data    # 1567 rows: label (-1/1) and Unix timestamp
  secom.names          # attribute metadata
```

After this completes, `run-secom`, `calibrate-secom`, `calibrate-secom-dsa`,
`render-non-intrusive-artifacts`, `render-unified-value-figure`, and `sbir-demo`
are all ready to run.

### Step 2 — Place PHM 2018 data (manual, ~2 GB)

The PHM 2018 dataset is not freely redistributable via automated download. You
must obtain it manually:

1. Go to the [PHM 2018 data challenge page](https://phmsociety.org/conference/annual-conference-of-the-phm-society/annual-conference-of-the-prognostics-and-health-management-society-2018-b/phm-data-challenge-6/)
2. Download the archive (Google Drive link on that page, ~2 GB `.tar.gz`)
3. Place the file **without extracting** at:

```
data/raw/phm2018/phm_data_challenge_2018.tar.gz
```

The crate extracts it on first use. Once placed, `probe-phm2018` and
`run-phm2018` are ready.

> **If you skip this step** and run `run-phm2018`, you will see:
> ```
> [phm2018] dataset not found — see --help or probe-phm2018 for placement instructions
> ```
> Use `probe-phm2018` at any time to check detection status.

### Working directory note

All subcommands resolve `data/` relative to the **current working directory**
when the binary is launched. If you install via `cargo install dsfb-semiconductor`,
always run the binary from the same directory where you placed (or will place)
your `data/` folder.

```bash
# Installed globally -- must cd to your data root first
cd ~/dsfb-workdir
dsfb-semiconductor fetch-secom
dsfb-semiconductor run-secom
```

---

## CLI quickstart

> **Prerequisite:** complete [Data preparation](#data-preparation) first.

```bash
# Fetch the SECOM dataset (automated download)
cargo run --release -- fetch-secom

# Run the full SECOM benchmark
cargo run --release -- run-secom

# Run deterministic SECOM parameter-grid calibration
cargo run --release -- calibrate-secom \
  --drift-window-grid 3,5 \
  --boundary-fraction-of-rho-grid 0.4,0.5 \
  --state-confirmation-steps-grid 1,2

# Run bounded DSA calibration grid (W, K, tau, m)
cargo run --release -- calibrate-secom-dsa

# Run PHM 2018 degradation benchmark
cargo run --release -- run-phm2018

# Probe PHM 2018 dataset availability
cargo run --release -- probe-phm2018

# Render non-intrusive architecture artifacts from a SECOM run
cargo run --release -- render-non-intrusive-artifacts \
  --run-dir output-dsfb-semiconductor/20260404_091433_203_dsfb-semiconductor_secom

# Render unified SECOM + PHM value figure
cargo run --release -- render-unified-value-figure \
  --secom-run-dir output-dsfb-semiconductor/<secom_run> \
  --phm-run-dir  output-dsfb-semiconductor/<phm_run>

# Run full SBIR demo (SECOM + calibration + DSA calibration + PHM in one shot)
cargo run --release -- sbir-demo

# Verify that the crate reproduces the paper headline numbers
cargo run --release -- paper-lock
# [paper-lock] episode count :   71  (expected 71)  OK
# [paper-lock] precision     :  80.3%  (expected >= 80%)  OK
# [paper-lock] recall        : 104/104  (expected 104/104)  OK
# [paper-lock] PASS -- headline numbers reproduced.
```

Key tunable flags for `run-secom`:

```
--healthy-pass-runs                    Healthy-window size (default 100)
--drift-window                         W_d for drift computation (default 5)
--envelope-sigma                       sigma multiplier for rho (default 3.0)
--boundary-fraction-of-rho             Threshold for Boundary zone (default 0.5)
--state-confirmation-steps             Hysteresis confirmation steps (default 2)
--persistent-state-steps               Persistent-mask steps (default 2)
--dsa-window                           W for DSA rolling window (default 5)
--dsa-persistence-runs                 K for DSA persistence gate (default 2)
--dsa-alert-tau                        tau for DSA score threshold (default 2.0)
--dsa-corroborating-feature-count-min  m for run-level alert (default 2)
```

---

## Supported datasets

See [Data preparation](#data-preparation) for setup instructions. This section
describes the datasets themselves.

### SECOM

Source: [UCI Machine Learning Repository — SECOM](https://archive.ics.uci.edu/dataset/179/secom)

- 590 numeric sensor columns
- 1567 wafer runs
- 104 failure-labeled runs (label = 1); 1463 pass runs (label = -1)
- ~40% of values are missing (NaN) — imputed by the healthy-window nominal mean
- Auto-downloaded by `fetch-secom`; cached at `data/raw/secom/`

SECOM is the primary benchmark for precision, recall, alarm burden, and episode
compression claims in the paper. All headline numbers come from SECOM.

### PHM 2018 ion mill etch

Source: [PHM 2018 data challenge](https://phmsociety.org/conference/annual-conference-of-the-phm-society/annual-conference-of-the-prognostics-and-health-management-society-2018-b/phm-data-challenge-6/)

- Continuous multi-sensor degradation trajectories for 10 tools, 2 datasets each
- 20 training runs + 5 test runs (25 CSVs total after extraction)
- Must be placed manually at `data/raw/phm2018/phm_data_challenge_2018.tar.gz`

PHM 2018 claims are bounded to degradation-oriented structure-emergence and
timing analysis against the run_energy_scalar_threshold baseline. No
burst-detection or burden-compression claim is made on PHM 2018.

---

## Output artifacts

All runs write to a timestamped directory that is never reused:

```
output-dsfb-semiconductor/<timestamp>_dsfb-semiconductor_<dataset>/
```

Key SECOM artifacts:

| File | Contents |
|------|----------|
| `benchmark_metrics.json` | Per-feature rho, thresholds, alarm counts, DSA scores |
| `episode_precision_metrics.json` | Episode count, precision, precision gain factor |
| `dsfb_traceability.json` | Full chain: Residual -> Sign -> Motif -> Grammar -> Semantic -> Policy |
| `parameter_manifest.json` | Every threshold computed from the healthy window |
| `dsa_parameter_manifest.json` | All DSA weights, tau, K, W, m |
| `engineering_report.pdf` | PDF report including all figures and artifact inventory |
| `run_bundle.zip` | Complete run directory as a ZIP |
| `figures/dsfb_unified_value_figure.png` | SECOM + PHM value figure |
| `figures/dsfb_non_intrusive_architecture.png` | Architecture diagram |
| `figures/drsc_dsa_combined.png` | Deterministic Residual Stateflow Chart + DSA overlay |
| `dsa_grid_results.csv` | Full bounded calibration grid results |
| `dsa_cohort_results.csv` | Per-cohort DSA results |
| `heuristics_bank.json` | Active operator-facing heuristics with governance fields |
| `non_intrusive_interface_spec.md` | Machine-readable non-intrusion guarantees |

The DRSC (Deterministic Residual Stateflow Chart) figure aligns four panels:

1. Normalized residual / drift / slew (r/rho, drift/delta, slew/zeta)
2. Confirmed persistent grammar state band
3. Feature-level DSA score with persistence-gated alert shading
4. Run-level threshold / EWMA / CUSUM / run-energy trigger timing

---

## no_std kernel

The computation kernel compiles without the standard library, requiring only
`alloc`. This enables deployment on Cortex-M microcontrollers, FPGAs, and RTOS
targets.

Kernel modules available without std:

```
config            process_context   units
sign              signs             nominal
residual          grammar           grammar::layer
syntax            semantics         policy
input
```

Verify no_std compilation for the Cortex-M4/M7 target:

```sh
cargo check --lib --no-default-features --target thumbv7em-none-eabi
```

---

## Reproducibility discipline

- All thresholds computed from the healthy window are saved to
  `parameter_manifest.json` at every run.
- All DSA weights, gate parameters, and corroboration rules are saved to
  `dsa_parameter_manifest.json`.
- Missing values are preserved at load time and imputed deterministically with
  the healthy-window nominal mean before residual construction.
- Repeated runs on identical inputs with identical parameters produce identical
  metrics, traces, and calibration rows (modulo timestamp in the output directory
  name).
- The `DsfbSignatureFile` type enables version-locked, diff-auditable
  configuration tracking across tool generations.
- The `paper-lock` CLI command verifies that the crate reproduces all three
  headline numbers from the paper with no code changes required.

---

## Caveats and non-claims

- This crate does not claim SEMI standards compliance or completed qualification.
- This crate does not claim universal superiority over SPC, EWMA/CUSUM,
  multivariate FDC, or ML baselines.
- The comparator set is bounded: univariate threshold, EWMA, CUSUM, run-energy
  scalar, and PCA T2/SPE. No ML anomaly baselines.
- The current nuisance analysis is a pass-run proxy on SECOM labels, not a
  fab-qualified false-alarm-rate study.
- SECOM is real semiconductor data, but it is not a deployment validation dataset.
- PHM 2018 claims are bounded to degradation-oriented timing analysis.
- PDF generation depends on `pdflatex` being present at runtime.
- The no_std kernel is verified against `thumbv7em-none-eabi`. No claim of
  `no_alloc`, SIMD, or parallel-acceleration support.
- Lead-time and density values are proxy metrics, not fab-qualified economic
  metrics.

---

## Citation

If you use this software in academic work, please cite both the software and
the associated paper:

**Paper:**

> de Beer, R. (2026). *DSFB Structural Semiotics Engine for Semiconductor
> Process Control -- A Deterministic Augmentation Layer for Typed Residual
> Interpretation for Fault Detection and Run-to-Run Variation in Advanced
> Manufacturing* (v1.0). Zenodo.
> https://doi.org/10.5281/zenodo.19413110

**BibTeX:**

```bibtex
@software{debeer2026dsfb,
  author    = {de Beer, R.},
  title     = {{DSFB Structural Semiotics Engine for Semiconductor Process Control}},
  subtitle  = {A Deterministic Augmentation Layer for Typed Residual Interpretation
               for Fault Detection and Run-to-Run Variation in Advanced Manufacturing},
  year      = {2026},
  version   = {1.0},
  publisher = {Zenodo},
  doi       = {10.5281/zenodo.19413110},
  url       = {https://doi.org/10.5281/zenodo.19413110}
}
```

---

## License

Apache-2.0. See [LICENSE](LICENSE).
