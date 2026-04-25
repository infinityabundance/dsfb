# FEMTO-ST PRONOSTIA — oracle protocol

Dataset: **FEMTO-ST / ENSMM PRONOSTIA accelerated bearing
degradation** (IEEE PHM 2012 Challenge).

## Provenance

- **Upstream:** <https://www.femto-st.fr/en/Research-departments/AS2M/Research-groups/PHM>
  and <https://github.com/wkzs111/phm-ieee-2012-data-challenge-dataset>
  (community mirror).
- **Test rig:** PRONOSTIA accelerated life-test platform, miniature
  ball bearings, accelerated via both mechanical overload and
  vibration-induced wear.
- **Sampling rates:** 25.6 kHz vibration (horizontal + vertical
  axes), 10 Hz temperature.
- **Corpus:** 17 bearings run to failure, split into three operating
  conditions.
- **Reference citation:** Nectoux P., Gouriveau R., Medjaher K.,
  Ramasso E., Chebel-Morello B., Zerhouni N., Varnier C.,
  "PRONOSTIA: An experimental platform for bearings accelerated
  degradation tests," *IEEE International Conference on Prognostics
  and Health Management*, 2012.

## Licence and redistribution

- **Licence:** free for academic research use with attribution
  (FEMTO-ST / ENSMM policy). Commercial use requires a separate
  arrangement with the institution.
- **This crate ships:** a manifest entry plus a 5-sample healthy +
  5-sample accelerated-aging illustrative slice in
  `src/datasets/femto_st.rs`. Full vibration time series are not
  redistributed.

## Fetch path

1. Download the PHM 2012 Challenge dataset from FEMTO-ST or the
   community mirror.
2. Extract to `crates/dsfb-robotics/data/femto_st/` preserving the
   per-bearing directory layout.
3. Compute a per-snapshot vibration health index (RMS, kurtosis,
   crest factor, spectral kurtosis). The adapter consumes the HI
   trajectory as input; HI construction is user-supplied.
4. Run `paper-lock femto_st`.

## Residual construction

For an HI trajectory `HI(k)`:

1. Calibrate `HI_calib = mean(HI_healthy_window)` via
   [`femto_st::Baseline::from_healthy`](../src/datasets/femto_st.rs).
2. DSFB residual: `r(k) = |HI(k) − HI_calib|`.

## DSFB bounded claim

DSFB does **not** claim lower RUL prediction error than the original
PHM 2012 Challenge submissions — those remain the state of the art
for that specific task. The bounded claim is *structural regime
precedence*: grammar-state transitions precede the RMS-threshold
alarm that the incumbent RUL estimator consumes. This is a
structural descriptor, not a superior RUL estimate.

## Reproducibility

- Headline numbers: `paper-lock femto_st` over the full challenge
  corpus under the pinned toolchain.
- Smoke-test: `paper-lock femto_st --fixture`.
