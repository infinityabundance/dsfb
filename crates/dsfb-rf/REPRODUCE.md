# REPRODUCE.md — `dsfb-rf` Reproducibility Guide

This document is the authoritative companion to the `dsfb-rf` crate and the
paper `dsfb_rf_v2.tex`. It describes **what is reproducible from the public
tree alone**, **what requires external datasets**, and **what is a smoke-test
stub rather than a paper claim**. The distinction matters: the paper's
broad prior-art posture is only defensible if the reader can tell these apart
without reading the code.

## 1. Toolchain (frozen)

| Component | Version | Source |
|---|---|---|
| Rust      | 1.85.1  | [rust-toolchain.toml](rust-toolchain.toml) |
| libhdf5   | ≥ 1.10  | system package (`libhdf5-dev` on Debian/Ubuntu) |
| Kani      | 0.52+   | <https://model-checking.github.io/kani/install-guide.html> |
| QEMU      | 7.2+    | `qemu-system-arm`, `qemu-system-riscv32` |
| Python    | 3.11+   | for `scripts/extract_radioml_slice.py` + `figures_all.py` |

Targets exercised in CI: `x86_64-unknown-linux-gnu`, `thumbv7em-none-eabihf`
(Cortex-M4F), `riscv32imac-unknown-none-elf`.

## 2. Dataset access and honesty disclosure

The v1.0.0 paper evaluation uses four **real** RF datasets: RadioML 2018.01a
(DeepSig), ORACLE (Northeastern GENESYS), POWDER (University of Utah / NSF
PAWR), and Colosseum (Northeastern RAFT / DARPA SC2). These datasets are
**not** tracked in this repository — their combined size exceeds 20 GB, and
several are gated by institutional or NDA-style licenses.

### 2.1 What ships inside the repo (real data, stratified slice)

[data/slices/](data/slices/) contains a **smoke-test asset**, not the paper's
evaluation dataset:

| File | Size | Provenance |
|---|---|---|
| `radioml_2018_slice.hdf5` | 1.85 MB | Stratified 240-capture slice of `GOLD_XYZ_OSC.0001_1024.hdf5` (24 modulations × 5 SNRs × 2 captures). Schema-preserving. |
| `deepsig_2018_snr30_slice.hdf5` | 395 KB | 100-capture head slice of legacy `DEEPSIG_2018_SNR30.hdf5`. Schema-preserving. |
| `SLICE_MANIFEST.json` | 16 KB | Parent SHA-256 + size, slice SHA-256 + size, stratification plan, license chain. |

**Purpose.** Enables `examples/radioml_hdf5.rs` and the `hdf5_loader` feature
to run on *real IQ samples* on a fresh clone, in CI, and under `cargo test`.

**What it does NOT support.** Headline paper numbers — Table 1 precision,
recall, ≥ 25× compression, Kani-bounded determinism sweep — are computed on
the **full** 20 GB GOLD file, not the slice. Running the `radioml_hdf5` example
on the slice yields trace-level output that demonstrates the HDF5 ingest path,
not a paper metric.

Regenerate the slice after any parent-file or stratification change:
```sh
python3 scripts/extract_radioml_slice.py \
    --gold   "/path/to/GOLD_XYZ_OSC.0001_1024.hdf5" \
    --deepsig "/path/to/DEEPSIG_2018_SNR30.hdf5" \
    --out    data/slices/
```

**Eight-dataset slice catalog (for the Colab reproducibility notebook).**
The companion notebook at [`colab/dsfb_rf_reproduce.ipynb`](colab/dsfb_rf_reproduce.ipynb)
exercises eight real public RF datasets as contextual slices. These are
prepared by [`scripts/prepare_slices.py`](scripts/prepare_slices.py) and
audited in [`data/slices/SLICE_MANIFEST.json`](data/slices/SLICE_MANIFEST.json):

| # | Slice                 | Provenance tier        | Real source                                                        |
|---|-----------------------|------------------------|--------------------------------------------------------------------|
| 1 | `radioml`             | `real-in-repo`         | DeepSig RadioML 2018.01a GOLD_XYZ_OSC stratified slice (see above) |
| 2 | `oracle`              | `real-local-zip`       | GENESYS ORACLE USRP X310 Raw IQ — head slice of `neu_m044q5210.zip` (user-downloaded from genesys-lab.org/oracle); preserves the parent's complex128-on-disk quirk |
| 3 | `powder`              | `real-local-zip`       | GENESYS POWDER LTE Band 7 capture — head slice of `neu_m046tb444.zip` (user-downloaded from genesys-lab.org/powder) |
| 4 | `tampere_gnss`        | `real-public`          | Tampere University GNSS Raw IQ — Zenodo 10.5281/zenodo.13846381, CC-BY 4.0, Wang/Sankari/Lohan/Valkama; fetched via HTTP Range |
| 5 | `coloran`             | `real-public`          | `wineslab/colosseum-oran-coloran-dataset` — `rome_static_medium/` bs.csv (KPI trace) |
| 6 | `coloran_commag`      | `real-public`          | `wineslab/colosseum-oran-commag-dataset` — `slice_mixed/` bs.csv (KPI + scheduling-policy trace) |
| 7 | `deepbeam`            | `real-local-file`      | Northeastern repo `neu:ww72bh952` — 8192-sample head slice of user-downloaded `neu_ww72bk394.h5` (59 GB parent, 11.06 B IQ rows); native NI schema `/iq (N,2)`, `/gain`, `/rx_beam`, `/tx_beam`; auth-gated mirror → user download required, parent identity pinned by first-4MiB SHA-256 |
| 8 | `deepsense_6g`        | `real-local-zip`       | DeepSense-6G Scenario 23 UAV mmWave — 1000-sample head slice of user-downloaded `scenario23_dev_w_resources.zip` (deepsense6g.net/scenarios/scenario-23). HDF5: `mmwave_power[time,beam]` float32 (N,64), `best_beam_index`, UAV telemetry (altitude, speed, pitch, roll, distance, height) |

Each slice is capped at ≤ 2 MB so the catalog stays git-committable. Every
proxy emission is stamped `[SYNTHETIC PROXY]` in cell output, SigMF/JSON
attributes, and `SLICE_MANIFEST.json` — never a substitute for a paper
result. Slices 5 and 6 are explicitly **non-IQ KPI annexes**; they are
never fed to the DSFB grammar FSM and exist for contextual breadth only.

Run the full catalog prep locally:
```sh
python3 scripts/prepare_slices.py          # try real, fall back to proxy
python3 scripts/prepare_slices.py --force  # regenerate from scratch
python3 scripts/prepare_slices.py --offline  # skip network; proxy-only
```

### 2.2 What requires external download (full evaluation dataset)

| Dataset | Full size | License | Access |
|---|---|---|---|
| RadioML 2018.01a | ~20 GB | CC BY-NC-SA 4.0 | <https://www.deepsig.ai/datasets> (registration wall) |
| ORACLE USRP B200 | ~100 GB | Non-commercial research | Northeastern GENESYS handle `hdl.handle.net/2047/D20324547` (authentication required as of 2026-04-20) |
| POWDER open-captures | varies | PAWR open research | <https://powderwireless.net/> (institutional PAWR account) |
| Colosseum RAFT | varies | DARPA SC2 research | Northeastern RAFT portal (institutional account) |

**Honest access status as of 2026-04-20.** At the time this document is
written, ORACLE, POWDER, and Colosseum are **not retrievable from fully
anonymous public URLs**. Institutional or research-program access is required.
We do not fabricate or regenerate those datasets synthetically; the examples
that carry their names (`oracle_usrp_b200.rs`, `nist_powder_playback.rs`,
`darpa_sc2_adversarial.rs`, `iqengine_diversity.rs`) run on *synthetic stubs*
when the real files are absent — and each such example now carries a loud
in-source banner stating exactly that (see §3).

**If you hold the real datasets**, place them under `data/` matching these
paths (paths are gitignored by design):
```
data/RML2018.01a.hdf5
data/RadioML HDF5/GOLD_XYZ_OSC.0001_1024.hdf5
data/oracle/<capture-name>.sigmf-{meta,data}
data/powder/<capture-name>.sigmf-{meta,data}
data/colosseum/<capture-name>.sigmf-{meta,data}
```

Then run the `paper-lock` binary to reproduce Table 1 row-for-row:
```sh
cargo run --bin paper-lock --features std,paper_lock,hdf5_loader
```

The `paper_lock.rs` tolerance gate (`|precision − 0.712| ≤ 0.005`,
`recall ≥ 96/102`) will enforce numerical identity with the paper within
bit-exact Q16.16 quantization — not statistical similarity.

### 2.3 Real-Dataset Figure Bank — 80 figures

A second, additive figure bank renders 10 DSFB structural exhibits per
slice × 8 real-world slices (RadioML, ORACLE, POWDER, Tampere GNSS,
ColO-RAN, ColO-RAN-commag, DeepBeam, DeepSense-6G). Every figure reads
from `data/slices/` and is captioned with the **upstream residual
producer** (matched filter / AGC / channel estimator / GNSS tracking
loop / scheduler EWMA / beamformer / beam-tracker) — DSFB is framed as
the structural interpreter of that producer's already-computed
residual, never as an alternative or "detects-earlier-than" claim.

Single-command reproduction (once `data/slices/` is populated via
`scripts/prepare_slices.py`):
```sh
cargo run --release --example generate_figures_real \
    --features std,serde,real_figures
```
Output: `../dsfb-rf-output/dsfb-rf-real-<ts>/figs/*.pdf` (80), merged
`dsfb-rf-all-real-figures.pdf`, `figure_data_real.json`, and an
artefacts zip. Missing slices emit loud `[SKIPPED — <slice> not
present]` banners; the remaining slices still render.

These 80 figures are **structural exhibits, not benchmark
reproductions**. No caption claims a modulation-classifier,
device-fingerprint, link-budget, spoofing-detection,
beam-selection-ML, or scheduling-policy benchmark; each slice's
non-claim list is embedded in `figure_data_real.json` and surfaced in
the corresponding caption footer.

## 3. Synthetic-stub examples (not paper claims)

Six examples carry real-dataset names but run on *synthetic* inputs when the
corresponding real files are absent. Each now opens with a visible banner and
a runtime `[SYNTHETIC STUB]` eprintln:

| Example | Stub purpose | Real-data upgrade path |
|---|---|---|
| `radioml_hdf5.rs` | Real RadioML IQ via slice (§2.1); still a proxy because DSFB wants receiver residual, not raw IQ amplitude. | Attach to a trained demodulator's residual stream. |
| `crawdad_interference.rs` | Synthetic WiFi/Bluetooth/Microwave interference patterns. | CRAWDAD captures (authenticated). |
| `atmospheric_fading_diag.rs` | Synthetic ionospheric scintillation traces. | ESA CEDAR archive (authenticated). |
| `gps_spoofing_detection.rs` | Synthetic spoofing-then-recovery trajectory. | UT Austin Radionavigation Lab (request). |
| `deep_space_metrology.rs` | Synthetic DSN occultation residual. | NASA PDS-Geosciences open archive. |
| `urban_multipath_prognosis.rs` | Synthetic Colosseum-flavoured multipath. | Colosseum RAFT (institutional). |

## 4. Figure-by-figure map (paper → artefact)

Full mapping is emitted at run time into the generated output folder. Run:
```sh
cargo run --example generate_figures_all --features std,serde
```

This produces `dsfb-rf-output/dsfb-rf-<YYYY-MM-DD_HH-MM-SS>/`:
- `figs/` — 50 figure PDFs + PNGs
- `dsfb-rf-all-figures.pdf` — merged PDF
- `figure_data.json` + `figure_data_all.json` — bit-exact engine data
- `dsfb-rf-<timestamp>-artifacts.zip` — the whole folder

**AGENTS.md rule §2 forbids writing to `paper/` from agents.** The pipeline
above never touches `paper/`.

## 5. Expected runtimes on reference workstation

| Task | Reference time (x86-64, Ryzen-class, 16 GB) |
|---|---|
| `cargo check --features std,serde` | ~15 s cold, ~2 s warm |
| `cargo test --all-features` | ~45 s |
| `cargo run --example generate_figures_all --features std,serde` | ~8 min |
| `cargo run --bin paper-lock --features std,paper_lock,hdf5_loader` (full GOLD) | ~22 min |
| `cargo kani --harness proof_grammar_evaluator_no_panic` | ~90 s |
| Full Kani suite (6 harnesses) | ~12 min |

## 6. Known degenerate inputs

See [tests/calibration_error_paths.rs](tests/calibration_error_paths.rs) (if
present) and the public error enum in [src/pipeline.rs](src/pipeline.rs):
- Empty healthy window.
- All-zero residual window (degenerate variance).
- NaN-contaminated window.
- Sub-floor-SNR window (< SNR floor in dB).

All four return a typed `Err` on the typed API path; the legacy panic path
is preserved behind `run_stage_iii_or_panic` for the `paper-lock` binary
whose pre-condition is enforced by the harness runner.

## 7. Hyperparameter sensitivity sweeps (L18 bound)

The paper fixes `W_pred = 5` and a single ρ envelope. To reproduce the
sensitivity surface:
```sh
cargo run --example wpred_sweep --features std
```
The output JSON contains a `(W_pred, ρ) → (precision, recall, latency)` grid
that the paper's Limitations §L18 discusses but does not chart in v1.0.0.

## 8. Verification gate

The companion script `scripts/verify_reproduction.sh` (when run in a checkout
that contains the real datasets) diffs the emitted JSON against committed
goldens within `paper_lock.rs` tolerances. For public clones without the real
datasets, it runs the `radioml_hdf5` example on the stratified slice from
§2.1 and confirms the ingest path produces a stable hash — **not** that
paper numbers are matched.

## 9. License + citation chain

- `dsfb-rf` crate and this file: Apache-2.0.
- RadioML 2018.01a slice: derivative work under CC BY-NC-SA 4.0 (DeepSig).
- GUM uncertainty budget references JCGM 100:2008 (freely redistributable).
- `paper/` is not shipped in this repository (gitignored).

Cite as: de Beer, R. "DSFB Structural Semiotics Engine for RF Signal
Monitoring" (2026), v1.0.0, companion paper `dsfb_rf_v2.tex`.
