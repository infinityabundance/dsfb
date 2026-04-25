# CWRU bearing — oracle protocol

Dataset: **Case Western Reserve University Bearing Data Center**.

## Provenance

- **Upstream:** <https://engineering.case.edu/bearingdatacenter>
- **Test rig:** 2 HP Reliance Electric motor, drive-end and fan-end
  6205-2RS JEM SKF deep-groove ball bearings.
- **Seeded faults:** inner-race, outer-race, and ball faults at
  diameters 0.007, 0.014, 0.021, 0.028 inch (EDM machining).
- **Loads:** 0, 1, 2, 3 HP.
- **Sampling rate:** 12 kHz and 48 kHz (the paper's CWRU runs use
  both where available).
- **Reference citation:** Smith W. A. and Randall R. B. "Rolling
  element bearing diagnostics using the Case Western Reserve
  University data: A benchmark study," *Mechanical Systems and
  Signal Processing*, 64–65, 2015, 100–131.

## Licence and redistribution

- **Licence:** The CWRU Bearing Data Center distributes the raw
  vibration records freely for academic use. There is no formal
  redistribution licence; derivative slices may be republished with
  attribution to the Data Center.
- **This crate ships:** a ≤ 4 KB stratified illustrative slice at
  `data/slices/cwru_slice.csv` with SHA-256 provenance in
  `data/slices/SLICE_MANIFEST.json` (populated in Phase 9). The
  full corpus is not redistributed.

## Fetch path

1. Visit <https://engineering.case.edu/bearingdatacenter> and
   download the `.mat` files for your regime of interest.
2. Place extracted files under
   `crates/dsfb-robotics/data/cwru/` following the directory layout
   documented in this file's §"Residual construction".
3. Re-run `paper-lock cwru` (without `--fixture`) — the adapter
   will pick up the full corpus automatically.

## Residual construction

For a healthy baseline signal `x_h(t)` and an observed signal
`x(t)`:

1. Apply the standard bearing envelope-analysis chain: Hilbert
   demodulation → band-pass filter centred on the BPFI harmonics
   (inner-race ball-pass frequency is the default; the adapter is
   parameterised) → RMS within a 1 024-sample analysis window.
2. The result is a per-window BPFI envelope amplitude `E_{BPFI}(k)`.
3. The DSFB residual is `r(k) = |E_{BPFI}(k) − μ_healthy|` where
   `μ_healthy` is calibrated from the healthy baseline via
   [`cwru::Baseline::from_healthy`](../src/datasets/cwru.rs).

## DSFB bounded claim

DSFB does **not** claim to detect bearing faults earlier than, or
more accurately than, the incumbent threshold alarm on the BPFI
amplitude. The bounded claim is: *review-surface compression and
episode precision over the BPFI-amplitude trajectory* — the
trajectory the threshold alarm discards between crossings. See
[`README.md`](../README.md) §"Non-Claims" and [`SAFETY.md`](../SAFETY.md)
§"Review posture".

## Reproducibility

- Headline numbers come from `paper-lock cwru` (real data) under
  the pinned toolchain in `rust-toolchain.toml`.
- Smoke-test numbers come from `paper-lock cwru --fixture`.
- The SHA-256 of the parent dataset bundle and the illustrative
  slice are recorded in `data/slices/SLICE_MANIFEST.json`.
