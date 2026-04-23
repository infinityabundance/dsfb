# `dsfb-rf` — Colab Reproducibility

Single-file deliverable: [`dsfb_rf_reproduce.ipynb`](dsfb_rf_reproduce.ipynb).

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-rf/colab/dsfb_rf_reproduce.ipynb)

## What it does

The notebook builds the `dsfb-rf` crate from source on every run, reproduces
all **67 paper figures** (`../dsfb-rf-output/dsfb-rf-<timestamp>/figs/*.pdf`
plus the merged `dsfb-rf-all-figures.pdf`), materialises an **eight-dataset
slice catalog** alongside, and packages everything into a single downloadable
zip.

Expected wall-clock on a free-tier Colab CPU runtime: **~15 min**
(≈ 10–14 min cargo build + figure generation, ≈ 1 min slice prep, rest is
zip packaging and download).

## Eight-dataset slice catalog

Prepared by [`../scripts/prepare_slices.py`](../scripts/prepare_slices.py)
and audited in [`../data/slices/SLICE_MANIFEST.json`](../data/slices/):

| # | Slice           | Provenance            | Real source                                                      |
|---|-----------------|-----------------------|------------------------------------------------------------------|
| 1 | RadioML         | `real-in-repo`        | DeepSig RadioML 2018.01a GOLD_XYZ_OSC (slice already extracted)  |
| 2 | ORACLE          | `real-local-zip`      | GENESYS ORACLE USRP X310 Raw IQ (neu_m044q5210.zip head slice)   |
| 3 | POWDER          | `real-local-zip`      | GENESYS POWDER LTE Band 7 capture (neu_m046tb444.zip head slice) |
| 4 | Tampere GNSS    | `real-public`         | Zenodo 10.5281/zenodo.13846381 (Wang/Sankari/Lohan/Valkama)      |
| 5 | ColO-RAN        | `real-public`         | wineslab/colosseum-oran-coloran-dataset — `rome_static_medium/`  |
| 6 | ColO-RAN-commag | `real-public`         | wineslab/colosseum-oran-commag-dataset — `slice_mixed/`          |
| 7 | DeepBeam        | `real-local-file`     | Northeastern repo neu:ww72bh952 — 8192-sample head slice of user-downloaded `neu_ww72bk394.h5` (NI mmWave I/Q, gain, rx_beam, tx_beam) |
| 8 | DeepSense-6G    | `real-local-zip`      | Scenario 23 UAV mmWave — 1000-sample head slice of user-downloaded `scenario23_dev_w_resources.zip` (deepsense6g.net/scenarios/scenario-23) |

Every proxy emission is loudly labelled `[SYNTHETIC PROXY — <name>]` in cell
output, HDF5/SigMF/CSV attributes, and `SLICE_MANIFEST.json`.

## What it does NOT do

- Does **not** render the paper.
- Does **not** run the `paper-lock` binary (needs the full 20 GB GOLD file;
  stays on local hardware per [`REPRODUCE.md`](../REPRODUCE.md) §2.2).
- Does **not** validate Table 1 numerically.
- Does **not** claim to reproduce any specific ORACLE/POWDER/DeepBeam/
  DeepSense/ColO-RAN/Tampere benchmark — those are contextual slice
  exhibits, not benchmark reproductions.

## Verifying locally without Colab

```bash
cd crates/dsfb-rf
python3 scripts/prepare_slices.py --force
cargo run --release --example generate_figures_all --features std,serde
ls ../dsfb-rf-output/dsfb-rf-*/figs/*.pdf | wc -l   # expect 67

# Optional — Real-Dataset Figure Bank (additive, feature-gated).
cargo run --release --example generate_figures_real \
    --features std,serde,real_figures
ls ../dsfb-rf-output/dsfb-rf-real-*/figs/*.pdf | wc -l   # expect 80
```

## Real-Dataset Figure Bank (additive, 80 figures)

The second, additive figure bank renders 10 DSFB structural exhibits per
slice × 8 real-world slices — every caption names the upstream residual
producer (matched filter / AGC / channel estimator / GNSS tracking
loop / scheduler EWMA / beamformer / beam-tracker) and frames DSFB as
the structural interpreter of that producer's already-computed
residual. These are structural exhibits, not benchmark reproductions;
no figure claims a modulation-classifier, device-fingerprint,
link-budget, spoofing, beam-selection, or scheduling-policy benchmark.

## Hygiene

- Cell outputs are cleared before commit. To re-clear after a local run:
  `jupyter nbconvert --clear-output --inplace colab/dsfb_rf_reproduce.ipynb`.
- No new Rust dependencies; the notebook is pure orchestration.
- `Cargo.toml` stays frozen at v1.0.0.
