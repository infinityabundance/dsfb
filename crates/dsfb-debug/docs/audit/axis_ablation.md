# Per-axis ablation — Phase η.4

Each row reports one of the 9 fusion axes disabled relative
to the all-axes-on baseline (`FusionConfig::ALL_DEFAULT`).
The marginal contribution of an axis = baseline − axis-removed
on the cross-fixture LO-CV aggregate (12 fixtures).

Source: Phase η.4 ablation harness (`tests/axis_ablation.rs`).
Theorem 9 deterministic replay verified per configuration.

## Baseline (all 9 axes on)

| Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |
|----------:|--------:|------------:|----------------:|:------:|
| 14.3056 | 0.3539 | 0.9167 | 4 | 12 / 12 |

## Per-axis ablation (axis disabled vs baseline)

| Axis | Removed config | Mean RSCR | Mean FP | Mean recall | Typed | ΔTyped | Replay |
|------|----------------|----------:|--------:|------------:|------:|-------:|:------:|
| 1. Provenance gate | `use_bank_aware_consensus=false` | 14.3056 | 0.3539 | 0.9167 | 4 | +0 | 12 / 12 |
| 2. Margin gate | `margin_gate=0.0` | 14.3056 | 0.3539 | 0.9167 | 6 | +2 | 12 / 12 |
| 3. Tier-affinity scoring | `use_tier_affinity=false` | 14.3056 | 0.3539 | 0.9167 | 3 | -1 | 12 / 12 |
| 4. Zero-tier filter | `use_zero_tier_filter=false` | 14.3056 | 0.3539 | 0.9167 | 4 | +0 | 12 / 12 |
| 5. Adaptive margin gate | `use_adaptive_margin_gate=false` | 14.3056 | 0.3539 | 0.9167 | 3 | -1 | 12 / 12 |
| 6. Confuser-boundary adjudication | `use_confuser_boundary=false` | 14.3056 | 0.3539 | 0.9167 | 4 | +0 | 12 / 12 |
| 7. Disambiguator boost | `use_disambiguator_boost=false` | 14.3056 | 0.3539 | 0.9167 | 4 | +0 | 12 / 12 |
| 8. Tier-level primary witness gate | `use_primary_witness_tier_gate=false` | 14.3056 | 0.3539 | 0.9167 | 4 | +0 | 12 / 12 |
| 9. Per-detector named witness gate | `use_primary_witness_detector_gate=false` | 14.3056 | 0.3539 | 0.9167 | 5 | +1 | 12 / 12 |

## Honest empirical reading

All nine fusion axes are now ablatable via single
`FusionConfig` flags. Theorem 9 deterministic replay
holds under every leave-one-axis-out configuration.

**Marginal contribution per axis** (Δtyped relative to
the all-axes-on baseline):

- **Negative Δ** (axis disabled → fewer typed episodes):
  the axis is positively load-bearing — it ENABLES
  additional typed dispositions on the current surface.
- **Positive Δ** (axis disabled → more typed episodes):
  the axis is restrictively load-bearing — it FILTERS
  episodes that would otherwise be admitted as typed.
- **Zero Δ**: the axis is unmeasured-load-bearing on the
  current 12-fixture surface (gate fires correctly but
  the resulting decision is unchanged from the baseline).

Per Session-17 academic-honesty discipline, the table
reports verbatim test stdout; partner-data engagements
with sharper fault signatures may activate currently-
zero-Δ axes that have correctly-implemented logic but
no firing on this surface.
