# Site Calibration — Operator Recipe

**What this is.** DSFB-Debug ships with a 32-motif heuristics bank
calibrated against the panel benchmarks (TrainTicket, AIOps Challenge
fault categories, Illinois SocialNetwork, Defects4J / BugsInPy /
PROMISE code-defect catalogs). When you point it at *your own*
observability stack, the canonical thresholds will likely not fit
your site's residual distribution: different services produce
different baseline variance, different drift cadence, different
slew profile. This document is the operator-side recipe for
site-calibrating DSFB-Debug — finding thresholds that match your
healthy-window distribution without mutating the canonical bank.

**Standing discipline (Sessions 1-19):** the canonical bank stays
hand-crafted; the calibration tool is **advisory only** — it
returns a `CalibrationReport` the operator reviews and selectively
applies. No automatic mutation. NIST SP 800-53 AU-3 audit-record
content (which percentile, which dataset, which healthy-window
sample count) is preserved for every recommendation.

---

## Quick start (5 minutes)

You have a healthy-window residual slice from your production
telemetry — a `&[f64]` buffer of length `num_signals × num_windows`
laid out row-major. To get site-specific thresholds:

```rust
use dsfb_debug::calibration::recommend_config_from_healthy;

// healthy_data: &[f64] — your residuals (no labelled faults expected)
// num_signals, num_windows: dimensions of the residual matrix
// percentile: 0.0..1.0 (0.90 = "fire on top 10% of healthy variation")
let report = recommend_config_from_healthy(
    healthy_data,
    num_signals,
    num_windows,
    0.90,  // operator-chosen percentile
);

// Inspect:
println!("Recommended slew_delta: {:.4}", report.config.slew_delta);
for rec in &report.motif_recommendations {
    println!(
        "{:?}: drift {:.4} / slew {:.4}",
        rec.motif,
        rec.recommended_drift_threshold,
        rec.recommended_slew_threshold,
    );
}

// Apply (operator decision — not automatic):
//   - Override `EngineConfig.slew_delta` per `report.config`
//   - Override per-motif thresholds in your local bank copy
//   - Or use `report` as input to a calibrated FusionConfig
```

---

## What the recipe does

The function `recommend_config_from_healthy`
([src/calibration.rs](../src/calibration.rs)) computes:

1. **Per-signal mean baseline** — the empirical mean of each
   signal's residual values across the healthy-window slice. This
   is the per-signal zero-line that the bank's drift / slew
   computations need.

2. **Per-(window, signal) residual norm, drift, slew** — re-derived
   from the residual matrix using the same arithmetic the engine
   would apply at runtime, but on the healthy slice only.

3. **Empirical distribution of healthy variation** — p50 / p90 /
   p99 of the residual norm, plus mean and p90 of drift and slew.
   Captured in the `HealthyStats` struct.

4. **Per-motif threshold recommendations** — each of the 32
   motifs gets a recommended `drift_threshold` and
   `slew_threshold` derived from the chosen percentile of the
   healthy-window empirical distribution. The default 90th
   percentile means: "the bank fires when drift / slew exceed
   the top 10% of what we observed under healthy operation."

5. **Recalibrated `slew_delta`** — the
   `EngineConfig.slew_delta` field is set to the chosen
   percentile of healthy slew, so the engine's per-(window,
   signal) slew computation uses your site's noise floor as its
   baseline.

The output is a `CalibrationReport` carrying:

- `config: EngineConfig` — the recalibrated engine config
- `motif_recommendations: Vec<MotifThresholdRecommendation>` — per-motif
  drift + slew thresholds, with the percentile used and dataset
  name as provenance
- `healthy_stats: HealthyStats` — empirical distribution summary

---

## Choosing the percentile

The percentile parameter is the **single most consequential
operator choice** in calibration. It controls the trade-off
between false-positive rate and detection sensitivity:

| Percentile | "Fire on…" | Operator profile |
|-----------:|------------|-------------------|
| 0.50 (p50) | top 50% of healthy variation | aggressive — catches mild drift, more FPs |
| 0.80 (p80) | top 20% | balanced for noisy production traffic |
| **0.90 (p90)** | **top 10%** | **default — conservative starting point** |
| 0.95 (p95) | top 5% | strict — only fires on substantial drift |
| 0.99 (p99) | top 1% | very strict — high-stakes only |

The DSFB-Debug recommendation: start at **0.90**, capture the
typed-confirmed episode count and FP rate on a known-healthy
slice, then tighten to 0.95 if false positives dominate or
relax to 0.80 if you suspect under-detection.

---

## Phase η.3 sensitivity-sweep findings (informative)

Session 18's sensitivity sweep
([docs/audit/sensitivity_sweep.md](audit/sensitivity_sweep.md))
varied five hyperparameters one-at-a-time across the 12-fixture
LO-CV surface. Verbatim findings:

| Parameter | Response on the 12-fixture surface |
|-----------|------------------------------------|
| **`min_consensus`** | **dominant lever** — typed-confirmed total moves 6 → 5 → 4 → 3 → 1 as N tightens 1 → 9; FP rate drops 0.354 → 0.205 → 0.159 → 0.132 |
| `margin_gate` | typed count responds 6 → 5 → 4 → 4 → 4 as gate tightens 0.10 → 0.50; FP unchanged |
| `scalar_k` | shallow response — only FP at 2.0 is meaningfully different (0.365 vs 0.354) |
| `cusum_h` | shallow response — FP varies 0.371 → 0.351 across {2, 3, 4, 5, 6} |
| `ewma_lambda` | shallow response — FP varies 0.354 → 0.351 across {0.05, 0.10, 0.20, 0.30, 0.40} |

**Honest empirical reading**: on the public-fixture surface, `min_consensus`
is the parameter that most affects fusion behaviour. The other four
are robust to ±1-2σ ranges; the canonical defaults
(`scalar_k=3.0`, `cusum_h=4.0`, `ewma_lambda=0.2`) work without
site-specific tuning on this surface. **Site-specific calibration
should focus on `min_consensus` first, then per-motif drift / slew
thresholds via this calibration recipe; the remaining parameters
are second-order on this evidence.**

The sensitivity numbers above are bounded to the 12-fixture
public-dataset surface. Partner-data engagements with sharper fault
signatures may surface different sensitivities; the `tests/sensitivity_sweep.rs`
harness re-runs end-to-end on fresh fixture sets in <10 minutes.

---

## Worked example — calibrating against a healthy slice

Suppose you have:

- `healthy_residuals.tsv` — a residual-projection-v2 TSV file with
  no labelled faults (the first N windows of a known-healthy span)
- 8 services, 200 windows = 1,600 cells

```rust
use dsfb_debug::adapters::residual_projection::parse_residual_projection;
use dsfb_debug::calibration::recommend_config_from_healthy;

let bytes = std::fs::read("healthy_residuals.tsv")?;
let matrix = parse_residual_projection(&bytes)?;

let report = recommend_config_from_healthy(
    &matrix.data,
    matrix.num_signals,
    matrix.num_windows,
    0.90,  // p90 default
);

// Operator review:
println!("Healthy p50 norm: {:.4}", report.healthy_stats.p50_residual_norm);
println!("Healthy p90 norm: {:.4}", report.healthy_stats.p90_residual_norm);
println!("Healthy p99 norm: {:.4}", report.healthy_stats.p99_residual_norm);
println!("Recommended slew_delta: {:.4}", report.config.slew_delta);
println!("Sample count: {}", report.healthy_stats.sample_count);

// Per-motif thresholds:
for rec in &report.motif_recommendations {
    println!(
        "{:?}: site drift {:.4} / canonical {:.4}; site slew {:.4} / canonical {:.4}",
        rec.motif,
        rec.recommended_drift_threshold, /* canonical */ 0.7,
        rec.recommended_slew_threshold,  /* canonical */ 0.5,
    );
}
```

**What you do with this:**

1. **Inspect the gap.** If your site's recommended thresholds are
   close to the canonical values, the bank is well-calibrated for
   you and no override is needed.

2. **Apply selectively.** If a motif's recommended threshold is
   meaningfully different from the canonical, override that motif
   only in your local bank copy. The unmodified bank stays the
   ground truth.

3. **Re-run on a fault-positive slice.** The healthy calibration
   sets a "noise floor"; the next step is to verify the calibrated
   bank still fires correctly on a labelled-fault slice from your
   production data.

4. **Track the provenance.** Every calibration recommendation
   carries `(percentile, dataset_name, sample_count)` — preserve
   these in your audit trail per NIST SP 800-53 AU-3.

---

## When calibration is NOT enough

If after calibration the FP rate is still too high or recall is
still too low:

- **Investigate the residual-projection layer first.** A noisy
  residual stream upstream of DSFB-Debug calibrates to a noisy
  bank. The calibration tool can't fix bad input.

- **Run the [Phase η.3 sensitivity sweep](audit/sensitivity_sweep.md)
  on your own fixtures.** Drop your fixture into `tests/sensitivity_sweep.rs`
  and re-run. The response curves on your data may differ from the
  public-fixture pattern documented above.

- **Review the [Phase η.4 axis ablation](audit/axis_ablation.md).**
  All 9 fusion axes are toggleable via `FusionConfig` flags; if a
  specific axis is over-aggressive on your data, ablate it and
  measure the delta.

- **Use Phase η.5 detector subset optimization
  ([docs/audit/detector_subset_opt.md](audit/detector_subset_opt.md)).**
  K=5 detectors achieved baseline recall on the public fixtures at
  9.5× lower FP rate than the full 203-detector ensemble. Your data
  may admit a similar minimal-sufficient subset; the tooling supports
  per-detector consensus weight overrides via
  `FusionConfig::detector_weight_overrides`.

- **Partner with us.** The public-dataset ceiling is tight; site-
  specific calibration with partnered deployment data is where the
  remaining empirical-rigor headroom lives.

---

## Implementation reference

- **Function**: `recommend_config_from_healthy` in
  [src/calibration.rs](../src/calibration.rs)
- **Sensitivity-sweep harness**: [tests/sensitivity_sweep.rs](../tests/sensitivity_sweep.rs)
- **Per-axis ablation harness**: [tests/axis_ablation.rs](../tests/axis_ablation.rs)
- **Detector subset opt harness**: [tests/detector_subset_opt.rs](../tests/detector_subset_opt.rs)
- **Phase η.3 ledger**: [docs/audit/sensitivity_sweep.md](audit/sensitivity_sweep.md)
- **Phase η.4 ledger**: [docs/audit/axis_ablation.md](audit/axis_ablation.md)
- **Phase η.5 ledger**: [docs/audit/detector_subset_opt.md](audit/detector_subset_opt.md)
- **Operator handbook**: [docs/operator_handbook.md](operator_handbook.md)
- **Onboarding**: [docs/onboarding.md](onboarding.md)
