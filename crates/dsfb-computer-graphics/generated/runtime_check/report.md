# DSFB Computer Graphics Evaluation Report

## Scope

This crate is a deterministic, crate-local evaluation artifact for temporal reuse supervision and fixed-budget adaptive sampling.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

What is demonstrated: host-realistic DSFB supervision, stronger heuristic baselines, multi-scenario behavior, ablation sensitivity, fixed-budget allocation comparisons, attachability surfaces, and architectural cost accounting.

What is not proven: production-scene generalization, measured GPU benchmark wins, engine deployment readiness, or universal superiority over strong heuristics.

## Scenario Suite

- `thin_reveal`: Moving occluder reveals thin vertical and diagonal structure on a deterministic patterned background.
- `fast_pan`: Faster occluder motion over a textured backdrop stresses motion disagreement, depth rejection, and neighborhood stability.
- `diagonal_reveal`: Diagonal subpixel structure on a high-contrast background stresses neighborhood clamping and thin-structure proxies.
- `contrast_pulse`: A bounded lighting change with no geometry reveal stresses false positives and differentiates residual-driven response from depth-only heuristics.
- `stability_holdout`: Static holdout case with no reveal event. Useful for verifying low false-positive intervention and bounded neutral behavior.

## Demo A Baselines and DSFB Variants

Baselines: fixed alpha, residual threshold, neighborhood clamp, depth/normal rejection, reactive-mask-style, and strong heuristic.

DSFB variants: visibility-assisted synthetic mode, host-realistic mode, no-visibility, no-thin, no-motion, no-grammar, residual-only, and trust-without-alpha-modulation.

## Canonical Headline

On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from 2.84366 for fixed alpha to 0.31904.

Against the strong heuristic baseline, host-realistic DSFB changed cumulative ROI MAE from 0.49435 to 0.31904; mixed outcomes are surfaced per scenario below.

Thin-Structure Reveal: host-realistic DSFB changed cumulative ROI MAE from 2.84366 (fixed alpha) and 0.49435 (strong heuristic) to 0.31904.

## Per-Scenario Outcome Summary

| Scenario | Expectation | Host vs fixed ROI gain | Host vs strong heuristic ROI gain | Non-ROI penalty vs fixed | Note |
| --- | --- | ---: | ---: | ---: | --- |
| Thin-Structure Reveal | BenefitExpected | 2.52462 | 0.17531 | -0.01025 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| Fast Lateral Reveal | BenefitExpected | 2.75422 | 0.14998 | -0.01991 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| Diagonal Subpixel Reveal | BenefitExpected | 2.47756 | 0.13199 | -0.00753 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| Contrast Pulse Stress | BenefitExpected | -0.00000 | -0.00000 | 0.00000 | Strong heuristic remains better on this scenario; the report surfaces that rather than hiding it. |
| Stability Holdout | NeutralExpected | -0.00000 | -0.00000 | 0.00000 | This is the honesty scenario: aggressive trust collapse is not expected to help, so false-positive response and non-ROI stability are the main evaluation criteria. |

## Ablation Summary

| Variant | Canonical cumulative ROI MAE | Canonical peak ROI MAE | Suite mean cumulative ROI MAE | Suite mean false-positive rate |
| --- | ---: | ---: | ---: | ---: |
| DSFB visibility-assisted | 0.32040 | 0.07198 | 0.20801 | 0.00819 |
| DSFB host-realistic | 0.31904 | 0.07198 | 0.20622 | 0.01268 |
| DSFB without visibility cue | 0.32040 | 0.07198 | 0.20801 | 0.01262 |
| DSFB without thin proxy | 0.49804 | 0.07198 | 0.31086 | 0.00109 |
| DSFB without motion disagreement | 0.31904 | 0.07198 | 0.20622 | 0.01262 |
| DSFB without grammar | 0.67995 | 0.19011 | 0.41330 | 0.01364 |
| DSFB residual-only | 0.15600 | 0.01978 | 0.09769 | 0.00067 |
| DSFB trust without alpha modulation | 3.59520 | 0.45485 | 2.25150 | 0.04947 |

## Demo B Fixed-Budget Study

On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

| Scenario | Imported trust ROI MAE | Combined heuristic ROI MAE | Uniform ROI MAE | Note |
| --- | ---: | ---: | ---: | --- |
| Thin-Structure Reveal | 0.03184 | 0.04977 | 0.17226 | Imported trust remains competitive under equal budget on this scenario. |
| Fast Lateral Reveal | 0.04760 | 0.08558 | 0.11886 | Imported trust remains competitive under equal budget on this scenario. |
| Diagonal Subpixel Reveal | 0.06273 | 0.03830 | 0.10784 | Combined heuristic remains stronger on this scenario, which is surfaced explicitly in the decision report. |
| Contrast Pulse Stress | 0.00008 | 0.00008 | 0.00008 | Imported trust remains competitive under equal budget on this scenario. |
| Stability Holdout | 0.00672 | 0.00363 | 0.00672 | Neutral case: guidance is not expected to produce a large win, so non-ROI penalties and concentration behavior matter more than raw ROI gain. |

## Attachability

The host integration surface is implemented around typed current color, history color, motion vectors, depth, normals, trust, alpha, intervention, and optional sampling-budget outputs. See `docs/integration_surface.md`.

## Cost Model

“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”

“The framework is compatible with tiled and asynchronous GPU execution.”

| Mode | Buffers | Approx ops / pixel | Approx reads / pixel | Approx writes / pixel |
| --- | ---: | ---: | ---: | ---: |
| Minimal mode | 3 | 20 | 6 | 3 |
| Host-realistic mode | 8 | 60 | 22 | 9 |
| Full research/debug mode | 12 | 66 | 24 | 16 |

## Aggregate Leaderboard

| Run | Mean rank | Mean cumulative ROI MAE | Mean non-ROI MAE | Benefit-scenario wins |
| --- | ---: | ---: | ---: | ---: |
| DSFB residual-only | 3.80 | 0.09769 | 0.00057 | 3 |
| DSFB host-realistic | 4.00 | 0.20622 | 0.00157 | 0 |
| DSFB visibility-assisted | 4.60 | 0.20801 | 0.00164 | 0 |
| Reactive-mask-style baseline | 4.80 | 0.30370 | 0.00179 | 0 |
| Residual-threshold baseline | 5.20 | 0.35697 | 0.00271 | 0 |
| DSFB without motion disagreement | 5.40 | 0.20622 | 0.00157 | 0 |
| Strong heuristic baseline | 5.40 | 0.29768 | 0.00052 | 0 |
| DSFB without visibility cue | 6.20 | 0.20801 | 0.00159 | 0 |
| Fixed-alpha baseline | 6.80 | 1.75750 | 0.00911 | 1 |
| Depth/normal rejection baseline | 7.20 | 0.55920 | 0.00290 | 0 |

## Remaining Blockers

- The scenario suite is still synthetic and does not prove production-scene generalization.
- The strong heuristic baseline remains competitive on some cases, so the crate supports evaluation diligence rather than universal win claims.
- Cost accounting is architectural and CPU-side within the crate; it is not a measured GPU benchmark.

## What Is Not Proven

- This report does not prove production-scene generalization.
- This report does not prove that DSFB beats every strong heuristic on every scenario.
- This report does not claim measured GPU hardware wins or production readiness.
