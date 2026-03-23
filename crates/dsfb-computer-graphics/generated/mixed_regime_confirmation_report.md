# Mixed-Regime Confirmation Report

**mixed_regime_status:** mixed_regime_confirmed_internal

**source:** internal synthetic scenario (`noisy_reprojection`)
**frame_index:** 6
**resolution:** 160×96
**roi_pixels:** 1658 / 15360 (10.8%)

## 1. Source of Case

Scenario: **noisy_reprojection** (`NoisyReprojection`). This scenario deliberately combines:
- Thin-structure disocclusion events at frame onset (aliasing pressure)
- Noisy motion reprojection that creates temporal frame-to-frame instability (variance/noise pressure)

Both signals are computed from actual pixel data at frame index 6 — not inferred or claimed without evidence.

## 2. Why Aliasing Pressure Is Present

**Signal:** spatial gradient magnitude in current color frame within ROI

| Metric | Value |
|--------|-------|
| ROI mean gradient magnitude | 0.16492 |
| Background mean gradient magnitude | 0.07126 |
| ROI enrichment ratio | 2.314× |
| Threshold for confirmation | 1.5× |
| **Aliasing confirmed** | **true** |

Interpretation: ROI pixels exhibit 2.31× higher spatial frequency (gradient magnitude) than non-ROI pixels. This reflects the thin-structure disocclusion event where high-frequency edge detail is revealed at the onset frame. A ratio ≥1.5× is classified as material aliasing pressure.

## 3. Why Variance/Noise Pressure Is Present

**Signal:** temporal variance (mean squared difference between current frame and reprojected history)

| Metric | Value |
|--------|-------|
| ROI mean temporal variance | 0.11268 |
| Background mean temporal variance | 0.03106 |
| ROI enrichment ratio | 3.628× |
| Threshold for confirmation | 1.3× |
| **Variance confirmed** | **true** |

**Motion vector enrichment (supporting):**
| Metric | Value |
|--------|-------|
| ROI mean MV magnitude | 0.8419 px |
| Background mean MV magnitude | 1.1424 px |
| ROI MV enrichment ratio | 0.737× |

Interpretation: ROI pixels exhibit 3.63× higher temporal frame-to-frame instability than non-ROI pixels. This reflects the noisy reprojection model where motion estimates have added stochastic error at the thin structure boundary, creating material variance/noise pressure co-active with the aliasing pressure above.

## 4. Confirmation Classification

**Classification: `mixed_regime_confirmed_internal`**

Both aliasing pressure (enrichment 2.31x >= threshold 1.5x) and variance/noise pressure (enrichment 3.63x >= threshold 1.3x) are materially active in the **same ROI** at the **same frame**. This is not a claim -- it is the direct output of computing both signals from the same pixel set.

## 5. Engine-Native Confirmation Status

**Engine-native mixed-regime: NOT CONFIRMED**

No real engine capture has been provided. The classification above is `internal-only`. A true engine-native mixed-regime case requires a renderer capture with a scene that naturally produces both aliasing and variance pressure in the same ROI (e.g., a thin wire or foliage element under noisy TAA reprojection). Engine-native confirmation remains pending.

## 6. What Still Remains Unproven

- Mixed-regime on real engine-native data (pending capture + appropriate scene)
- Renderer-specific noise sources (e.g., blue-noise dither patterns) not evaluated
- Sub-pixel jitter interaction with aliasing is not separately quantified

## What Is Not Proven

- Engine-native mixed-regime confirmation (internal synthetic only)
- Renderer-specific variance sources not evaluated

## Remaining Blockers

- **EXTERNAL**: Engine-native mixed-regime requires real capture with appropriate scene.
- **INTERNAL** (resolved): Internal confirmation computed from actual signal values.
