# DSFB-Debug — Incumbent-Comparison Protocol

Methodology for fairly comparing DSFB-Debug against incumbent
trace-anomaly-detection methods on the same vendored fixture. **No
numerical results are claimed in this document** — the protocol is
deliverable today; the per-detector numerical fill-in is a follow-up
session that requires running each baseline detector on the same
slice and measuring against the same ground truth.

## Detector candidates

For each comparison, run all of the following on the same fixture's
residual matrix and the same fault labels:

1. **Scalar threshold detector.** Per signal, fire an alert when the
   value exceeds `mean_healthy + k·sigma_healthy`. `k = 3` is the
   conventional 3-sigma envelope. This is the strawman that DSFB-Debug
   replaces structurally.
2. **CUSUM (cumulative sum).** Page (1954). Per signal, accumulate
   `Σ (x - target)`; alert when the running sum exceeds a threshold.
   Adaptive to small persistent drifts that scalar threshold misses.
3. **EWMA (exponentially-weighted moving average).** Roberts (1959).
   Per signal, alert when EWMA crosses a threshold. More responsive
   than scalar threshold; less sensitive to brief spikes than CUSUM.
4. **Isolation Forest.** Liu et al. (2008). Tree-based outlier
   detection on the per-window feature vector. Captures multivariate
   anomalies the per-signal detectors miss.
5. **Dataset's labeled-anomaly baseline.** Where the dataset itself
   ships a baseline detector (TADBench publishes labels + a
   reference-detector evaluation framework; AIOps Challenge evaluates
   submissions against organiser-defined criteria), use that as the
   incumbent.

## Common metric set

For every detector × fixture combination, report:

- **RSCR** (Review Surface Compression Ratio) = raw alerts / typed
  episodes. Where a detector doesn't produce typed episodes, treat
  RSCR = 1 (no compression).
- **Fault recall** = fraction of labeled fault windows captured by at
  least one alert / episode within ±W_pred windows.
- **Episode precision** (DSFB-Debug-specific) = fraction of episodes
  followed by a labeled fault within W_pred windows.
- **Mean time from boundary to violation (MTBV)** = average wall-clock
  delay between earliest boundary indication and the labeled fault.
- **Clean-window false-positive rate** = false alerts (or false
  episodes for DSFB-Debug) per healthy window outside fault ranges.
- **Wall-clock processing time** per evaluation, excluding upstream
  fetch / projection.
- **Determinism** (yes / no) = does the detector produce identical
  output on identical bytes? DSFB-Debug: yes (Theorem 9). Most ML
  baselines: no (model variance, batch ordering, GPU non-associativity).

## Fairness protocol

Mandatory invariants per comparison:

1. **Identical residual matrix.** All detectors receive the same per-
   window per-signal residual matrix produced by the projection step.
   No detector gets per-detector preprocessing advantages.
2. **Identical fault labels.** The label set is fixed; no detector
   gets to retune labels mid-run.
3. **Identical evaluation windows.** Healthy-baseline / evaluation /
   prediction-window split is constant across detectors.
4. **No post-hoc threshold tuning on the eval split.** Each detector's
   threshold is fixed before observing eval-split results — fitted on
   healthy baseline only. The DSFB-Debug bank thresholds are the
   hand-curated ones in `src/heuristics_bank.rs`; re-tuning at site
   uses the calibration tool but only on the healthy slice.
5. **Single-pass determinism check.** Each detector runs twice on
   the same input; identical numerical output is required to claim
   the determinism row. ML detectors typically fail this; DSFB-Debug
   passes it by Theorem 9.

## Results matrix shape

Per fixture, fill a results matrix:

| Detector | RSCR | Fault recall | Episode precision | Clean-window FP rate | Wall-clock | Deterministic |
|----------|-----:|-------------:|------------------:|---------------------:|-----------:|--------------:|
| Scalar threshold | … | … | n/a | … | … | yes (trivial) |
| CUSUM | … | … | n/a | … | … | yes |
| EWMA | … | … | n/a | … | … | yes |
| Isolation Forest | … | … | n/a | … | … | typically no |
| Dataset baseline | … | … | … | … | … | per-baseline |
| **DSFB-Debug** | … | … | … | … | … | **yes** |

The "n/a" columns reflect the fact that flat-detector baselines do
not produce typed episodes; the comparison is honest about which
columns are DSFB-Debug-specific.

## Sensitivity analysis

For each detector that has a tunable threshold:

- Sweep the threshold from a permissive value to a strict value.
- Report a precision-recall curve (or, where false positives are the
  dominant concern, an FP-rate vs fault-recall curve).
- Record the threshold value at which the detector matches DSFB-Debug
  on fault recall and report the corresponding FP rate.
- Conversely, record the threshold at which the detector matches
  DSFB-Debug on FP rate and report the corresponding fault recall.

The headline claim DSFB-Debug aims to support empirically: at the
point where the comparison detector matches DSFB-Debug on fault
recall, DSFB-Debug has a *lower* clean-window false-episode rate
(because its persistence + correlation gates filter transient
boundary touches that flat detectors trip on). This is a falsifiable
claim contingent on the per-detector measurement.

## Result-publication discipline

When numerical findings exist:

- Cite each detector's implementation by version (e.g. `scikit-learn
  1.4.2 IsolationForest`, `python-cusum 0.1.0`, `dsfb-debug 0.2.0
  paper-lock`).
- Cite the fixture by manifest key + fixture SHA-256.
- Report each detector's threshold (or hyper-parameter set).
- Number-by-number; no rounding, no smoothing, no extrapolation.
- Sensitivity ranges where applicable; not single-point estimates.

## Phase II reservation

Per the academic-honesty discipline, this document is the protocol
only. The numerical fill-in lands in the empirical companion paper
once at least one fixture has been:

1. Vendored end-to-end (TrainTicket-Anomaly F-11 has been; see
   `data/MANIFEST.toml`).
2. Run through DSFB-Debug to capture the JSON metric block.
3. Run through each named baseline detector with matching invariants.
4. Cross-checked across two independent runs for determinism / variance.

The current artefact reports DSFB-Debug's row of the matrix on the
F-11 fixture: RSCR = 3.67×, deterministic_replay_holds = true,
clean-window false-episode rate = 0.7%. Baseline-detector rows are
deferred until the comparison runs are conducted. No claim of
relative superiority is made today.
