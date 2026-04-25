# IMS run-to-failure — oracle protocol

Dataset: **IMS (Intelligent Maintenance Systems) bearing run-to-failure**,
NASA Prognostics Data Repository.

## Provenance

- **Upstream:** <https://www.nasa.gov/intelligent-systems-division/discovery-and-systems-health/pcoe/pcoe-data-set-repository/>
  (select "Bearing Data Set").
- **Publisher:** NASA Ames PCoE / University of Cincinnati Center
  for Intelligent Maintenance Systems.
- **Test rig:** four Rexnord ZA-2115 double-row bearings on a
  shaft rotating at 2 000 rpm under a constant 6 000 lbs radial load.
- **Experiments:** three test-to-failure runs, 35-day total run
  time, bearings run until fault — no seeded faults, genuine
  accelerated degradation.
- **Sampling rate:** 20 kHz, 10-minute snapshots taken every 10 minutes.
- **Reference citation:** Lee J., Qiu H., Yu G., Lin J. et al.,
  "Rexnord Technical Services: Bearing Data Set," IMS Report, 2007;
  Qiu H., Lee J., Lin J., Yu G., "Wavelet Filter-based Weak
  Signature Detection Method and its Application on Rolling Element
  Bearing Prognostics," *Journal of Sound and Vibration*, 289(4–5),
  2006, 1066–1090.

## Licence and redistribution

- **Licence:** NASA-published, free for academic and commercial use
  with attribution (NASA data sharing policy).
- **This crate ships:** a ≤ 4 KB stratified health-index slice at
  `data/slices/ims_slice.csv` with SHA-256 provenance. The full
  20 kHz vibration snapshots are not redistributed.

## Fetch path

1. Register on the NASA PCoE repository and download the IMS
   Bearing Data Set.
2. Extract to `crates/dsfb-robotics/data/ims/` preserving the
   3-experiment directory layout.
3. Compute a per-snapshot health index (RMS, kurtosis, or an
   equivalent scalar). This crate's adapter takes the HI trajectory
   as input; HI construction is user-supplied because practitioners
   choose different HI formulas depending on their feature engineering
   pipeline. Reference Qiu et al. 2006 for the wavelet-kurtosis HI
   the paper headline numbers use.
4. Run `paper-lock ims` against the full trajectory.

## Residual construction

For an HI trajectory `HI(k)`:

1. Calibrate `HI_nominal = mean(HI_healthy_window)` from the first
   ≈ 20 % of snapshots (Stage III protocol).
2. DSFB residual: `r(k) = |HI(k) − HI_nominal|` — see
   [`ims::Baseline::residual_norm`](../src/datasets/ims.rs).

## DSFB bounded claim

DSFB does **not** claim an earlier remaining-useful-life prediction
than the PHM 2012-style RUL estimators. The bounded claim is
**grammar-state timing precedes RMS-threshold crossing** — a
structural descriptor of the HI trajectory that the RUL estimator
collapses to a scalar and discards. See [`README.md`](../README.md)
§"Non-Claims" for the full non-RUL disclaimer.

## Reproducibility

- Headline numbers: `paper-lock ims` over the full corpus under the
  pinned toolchain.
- Smoke-test: `paper-lock ims --fixture` uses the 5-sample healthy +
  6-sample trajectory embedded in `src/datasets/ims.rs`.
