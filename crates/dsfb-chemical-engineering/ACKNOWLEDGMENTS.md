# Acknowledgments & dataset credits

DSFB-Chemical-Engineering is a read-only residual-interpretation layer; it builds entirely on datasets, benchmarks, and
process models openly shared by others. **All credit for the underlying data belongs to its authors and providers, and
we thank them.** Above all, we thank the countless chemical engineers and process scientists whose accumulated work this
one builds upon — we see as far as we do only by standing on the shoulders of giants, and without them this paper and
this work would not be possible. Per-dataset provenance — source URL, license, citation, retrieval date, and the committed-slice
SHA-256 — is in [`crates/dsfb-chemical-engineering-edge/data/MANIFEST.toml`](crates/dsfb-chemical-engineering-edge/data/MANIFEST.toml).

## Controlled-access testbeds — iTrust / SUTD (with thanks, and a request-not-redistribution posture)

We gratefully acknowledge **iTrust, Centre for Research in Cyber Security, Singapore University of Technology and
Design (SUTD)** for the **SWaT** and **WADI** testbeds.

- iTrust: <https://itrust.sutd.edu.sg/>
- iTrust datasets (request page): <https://itrust.sutd.edu.sg/itrust-labs_datasets/>
- SWaT — Secure Water Treatment: Goh, Adepu, Junejo & Mathur (2016).
- WADI — Water Distribution: Ahmed, Palleti & Mathur (2017).

These are **controlled-access** under an iTrust data-use agreement. We **requested official iTrust/SUTD access and that
request is pending their reply** at the time of release. This repository therefore ships **only** a clearly-labelled
synthetic SWaT stand-in, recipe scripts, metadata-only sidecars, and aggregate-only / non-reconstructable figures — it
**does not redistribute** any raw rows, processed rows, attack lists, witness CSVs, or reconstructable windows. Any
mirror-derived local evaluation here is treated as provisional and aggregate-only. **Users must request SWaT/WADI
directly from iTrust/SUTD** at the link above and comply with the iTrust agreement (credit iTrust/SUTD; notify iTrust on
publication; no sharing).

## Water-distribution attack benchmark — BATADAL (with thanks)

We gratefully acknowledge the organisers of the **Battle of the Attack Detection Algorithms (BATADAL)** — Riccardo
Taormina, Stefano Galelli, Avi Ostfeld, and the full competition committee — for the C-Town water-distribution attack
benchmark.

- BATADAL: <https://www.batadal.net/>
- Taormina, Galelli, Tippenhauer, Salomons, Ostfeld, et al. (2018), *Battle of the Attack Detection Algorithms:
  Disclosing cyber attacks on water distribution networks*, **J. Water Resources Planning and Management**.

## Open ML/process datasets — UCI Machine Learning Repository (with thanks)

We gratefully acknowledge the **UCI Machine Learning Repository** and the original dataset authors for the following
openly-licensed (CC BY 4.0) datasets.

- UCI Machine Learning Repository: <https://archive.ics.uci.edu/>
- **air_quality_multisensor** — De Vito, Massera, Piga, Martinotto & Di Francia (2008), *On field calibration of an
  electronic nose…*, **Sensors & Actuators B**.
- **gas_sensor_array_drift** — Vergara, Vembu, Ayhan, Ryan, Homer & Huerta (2012), *Chemical gas sensor drift
  compensation…*, **Sensors & Actuators B**.
- **secom_semiconductor** — McCann & Johnston (2008), SECOM semiconductor manufacturing process dataset.
- **steel_plates_faults** — Semeion Research Center of Sciences of Communication, via the UCI Steel Plates Faults
  dataset.
- **wine_quality_red / white** — Cortez, Cerdeira, Almeida, Matos & Reis (2009), *Modeling wine preferences by data
  mining from physicochemical properties*, **Decision Support Systems**.

## NIR calibration-transfer benchmark — Eigenvector Research (with thanks)

We gratefully acknowledge **Eigenvector Research, Inc.** for the publicly-shared Corn NIR calibration-transfer
(instrument-standardisation) benchmark used here as `corn_nir_m5 / mp5 / mp6`.

- Eigenvector data sets: <https://eigenvector.com/resources/data-sets/>
- Eigenvector Corn NIR standardisation benchmark (m5 / mp5 / mp6 spectrometers).

## NIR meat spectroscopy — Tecator / StatLib, via OpenML (with thanks)

We gratefully acknowledge the **Tecator** dataset (originally distributed via StatLib) and **OpenML** for hosting it, as
`tecator_nir_meat`.

- OpenML dataset 505: <https://www.openml.org/d/505>
- Borggaard & Thodberg (1992), *Optimal minimal neural interpretation of spectra*, **Analytical Chemistry**.

## Tennessee Eastman process — Downs & Vogel; Braatz-group distribution (with thanks)

We gratefully acknowledge **J. J. Downs and E. F. Vogel** for the Tennessee Eastman process (TEP) challenge problem, the
**Braatz research group** for the widely-used MATLAB distribution, and **C. Rieth et al.** for the extended fault set, as
`tennessee_eastman_idv01 / 04 / 06 / 13 / 14`.

- Braatz-group TEP distribution: <https://github.com/camaramm/tennessee-eastman-profBraatz>
- Downs & Vogel (1993), *A plant-wide industrial process control problem*, **Computers & Chemical Engineering**.
- Chiang, Russell & Braatz (2001), *Fault Detection and Diagnosis in Industrial Systems*, Springer.
- Rieth, Amsel, Tran & Cook (2017), TEP simulation dataset, **Harvard Dataverse**.

## Wastewater-treatment benchmark model — IWA BSM1 (with thanks)

We gratefully acknowledge the **IWA Task Group on Benchmarking of Control Strategies for Wastewater Treatment Plants** for
the Benchmark Simulation Model no. 1 (BSM1), which we **re-implement** here as `bsm1_wastewater` (no third-party bytes are
vendored).

- IWA benchmark models: <https://github.com/wwtmodels/Benchmark-Simulation-Models>
- Alex, Benedetti, Copp, Gernaey, Jeppsson, Nopens, Pons, Rieger, Rosen, Steyer, Vanrolleghem & Winkler, IWA Benchmark
  Simulation Model no. 1 (BSM1).

## Industrial-scale penicillin fed-batch — IndPenSim (with thanks)

We gratefully acknowledge **Stephen Goldrick and colleagues** for IndPenSim, the industrial-scale penicillin fed-batch
simulation, which we **re-implement** here as `penicillin_fedbatch` (no third-party bytes are vendored).

- IndPenSim data: <https://data.mendeley.com/datasets/pdnjz7zz5x>
- Goldrick, Ştefan, Lovett, Montague & Lennox (2015), *The development of an industrial-scale fed-batch fermentation
  simulation*, **Journal of Biotechnology**; and the 2019 IndPenSim release.

## Textbook + FDI benchmark process models (with thanks)

We gratefully acknowledge the authors of the following standard process-engineering benchmark models, which we
**re-implement** from their published descriptions (no third-party bytes are vendored).

- **cstr_reactor** — Seborg, Edgar, Mellichamp & Doyle, *Process Dynamics and Control* (the textbook CSTR benchmark).
- **three_tank** — the **DTS200 / Amira** three-tank fault-detection-and-isolation (FDI) benchmark family.

## All datasets & benchmarks (full credit list)

| Dataset(s) in this repo | Provider / authority | Link | License | Citation |
|---|---|---|---|---|
| SWaT (stand-in) | iTrust / SUTD | <https://itrust.sutd.edu.sg/itrust-labs_datasets/> | iTrust DUA (not redistributed) | Goh et al. 2016 |
| WADI | iTrust / SUTD | <https://itrust.sutd.edu.sg/itrust-labs_datasets/> | iTrust DUA (not redistributed) | Ahmed et al. 2017 |
| BATADAL (C-Town) | BATADAL competition (Taormina, Galelli, Ostfeld, et al.) | <https://www.batadal.net/> | competition benchmark | Taormina et al. 2018, *J. Water Res. Plng. Mgmt.* |
| air_quality_multisensor | UCI Machine Learning Repository | <https://archive.ics.uci.edu/> | CC BY 4.0 | De Vito et al. 2008, *Sensors & Actuators B* |
| gas_sensor_array_drift | UCI Machine Learning Repository | <https://archive.ics.uci.edu/> | CC BY 4.0 | Vergara et al. 2012, *Sensors & Actuators B* |
| secom_semiconductor | UCI Machine Learning Repository | <https://archive.ics.uci.edu/> | CC BY 4.0 | McCann & Johnston 2008 |
| steel_plates_faults | UCI Machine Learning Repository (Semeion) | <https://archive.ics.uci.edu/> | CC BY 4.0 | Semeion Research Center; UCI Steel Plates Faults |
| wine_quality_red / white | UCI Machine Learning Repository | <https://archive.ics.uci.edu/> | CC BY 4.0 | Cortez et al. 2009, *Decision Support Systems* |
| corn_nir_m5 / mp5 / mp6 | Eigenvector Research, Inc. | <https://eigenvector.com/resources/data-sets/> | public (Eigenvector) | Eigenvector Corn NIR standardisation benchmark |
| tecator_nir_meat | Tecator / StatLib, via OpenML | <https://www.openml.org/d/505> | public | Borggaard & Thodberg 1992, *Anal. Chem.* |
| tennessee_eastman_idv01/04/06/13/14 | Downs & Vogel TEP; Braatz-group distribution; extended set on Harvard Dataverse | <https://github.com/camaramm/tennessee-eastman-profBraatz> | public | Downs & Vogel 1993; Chiang, Russell & Braatz 2001; Rieth et al. 2017 |
| bsm1_wastewater (model) | IWA Task Group (BSM1) | <https://github.com/wwtmodels/Benchmark-Simulation-Models> | n/a (re-implemented model) | Alex et al., IWA Benchmark Simulation Model no. 1 |
| penicillin_fedbatch (model) | IndPenSim (Goldrick et al.) | <https://data.mendeley.com/datasets/pdnjz7zz5x> | n/a (re-implemented model) | Goldrick et al. 2015/2019, IndPenSim |
| cstr_reactor (model) | textbook benchmark | — | n/a (generated physics) | Seborg et al., *Process Dynamics & Control* |
| three_tank (model) | DTS200 / Amira three-tank FDI benchmark family | — | n/a (generated physics) | DTS200 / Amira three-tank FDI benchmark |

Links are the providers' canonical pages; the exact committed download URLs and per-slice SHA-256 are in
`MANIFEST.toml`. Datasets marked *(model)* are re-implemented from the cited process models — no third-party bytes are
vendored. UCI repository entries are reached from the repository home above by dataset name; the verified per-dataset
download URLs are recorded in `MANIFEST.toml`.

## Detector & method originators — the science this layer reads (with thanks)

DSFB-Chemical-Engineering executes no new detector: it is a read-only layer over the residuals, scores, distances, and
reconstruction errors that established chemometric and statistical-process-control methods already emit. We gratefully
acknowledge the scientists and engineers who invented those methods. The detectors the pipeline actually runs (the 18
*executed* records in [`dsfb-chemical-engineering-atlas`](crates/dsfb-chemical-engineering-atlas/src/detector.rs)) and
the foundational methods they build on are due to:

**Foundations.** Principal Component Analysis — **Karl Pearson** (1901) and **Harold Hotelling** (1933); Partial Least
Squares for chemometrics — **Herman Wold** and **Svante Wold**; the statistical distance — **Prasanta Chandra
Mahalanobis** (1936).

**Classical multivariate SPC**
- **Shewhart control chart / 3-sigma rule** — **Walter A. Shewhart**.
- **Robust location/scale (median–MAD, Hampel filter)** — **Frank R. Hampel**.
- **Hotelling's $T^2$** (score-space monitoring index) — **Harold Hotelling**.
- **SPE / Q-residual statistic** — **J. Edward Jackson & Govind S. Mudholkar** (1979).

**Dynamic / temporal**
- **EWMA** — **S. W. Roberts** (1959).
- **CUSUM** — **E. S. Page** (1954).
- **Page–Hinkley change detection** — **E. S. Page & D. V. Hinkley**.
- **Mann–Kendall nonparametric trend test** — **Henry B. Mann & Maurice G. Kendall**.
- **MEWMA (multivariate EWMA)** — **Cynthia A. Lowry, William H. Woodall, Charles W. Champ & Steven E. Rigdon** (1992).
- **Dynamic PCA (lag-augmented)** — **Wenfu Ku, Robert H. Storer & Christos Georgakis** (1995).
- **MOSUM (moving-sum control)** — **Peter Bauer & Peter Hackl** (1978).

**Nonlinear / distributional**
- **Kolmogorov–Smirnov two-sample test** — **Andrey Kolmogorov & Nikolai Smirnov**.
- **Maximum Mean Discrepancy (kernel two-sample)** — **Arthur Gretton, Karsten Borgwardt, Malte Rasch, Bernhard
  Schölkopf & Alexander Smola** (2012).
- **Spectral (Shannon) entropy** — **C. E. Shannon** (1948), applied to the residual power spectrum.
- **Distance-/density-based anomaly scoring (kNN)** — the nearest-neighbour rule of **Evelyn Fix & Joseph Hodges**
  (1951) and **Thomas Cover & Peter Hart** (1967); distance-based outlier detection — **Edwin Knorr & Raymond Ng**
  (1998) and **Sridhar Ramaswamy, Rajeev Rastogi & Kyuseok Shim** (2000).
- **Population Stability Index** — the population-drift index of the credit-scoring literature, rooted in the relative
  entropy (Kullback–Leibler divergence) of **Solomon Kullback & Richard Leibler** (1951).

**Process-structure isolation.** The `co_drift` and `sensor_bias` witnesses are DSFB process-structure constructs
grounded in the multivariate-residual contribution-plot / fault-isolation lineage — **Theodora Kourti & John F.
MacGregor**; **Robert L. Miller, Robert E. Swanson & Carlos F. Heckler** (1998); **Johan A. Westerhuis, Stephen P.
Gurden & Age K. Smilde** (2000).

The wider *catalogued* prior-art surface — Kernel PCA (**Bernhard Schölkopf, Alexander Smola & Klaus-Robert Müller**),
Independent Component Analysis (**Pierre Comon**; **Aapo Hyvärinen & Erkki Oja**), autoencoders, and others — is recorded
with source references in the detector corpus (`corpus/chemometric_atlas.toml`) and the atlas records.

If we have mis-credited or under-credited any provider, method, or author, that is an error to fix — please open an issue.
