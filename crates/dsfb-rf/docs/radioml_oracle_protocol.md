# Stage III Fixed Evaluation Protocol: RadioML 2018.01a + ORACLE

**Document status:** Normative reference for the Stage III evaluation reported
in de Beer (2026). This document defines the exact protocol, dataset access
procedure, parameter fixings, and negative-control scope.

---

## Scope and Non-Claims

This protocol covers **Stage III: public-data read-only evaluation only**.

- **No live receiver integration** is claimed or performed.
- **No ITAR-controlled evaluation** is claimed or performed.
- **No operational deployment result** is claimed or included.
- Results are bounded to the two datasets and parameter fixings below.
- No extrapolation to other signal environments, hardware, or SNR ranges
  is made from these results.

---

## Dataset 1: DeepSig RadioML 2018.01a

### What It Is

A publicly available synthetic IQ dataset (O'Shea and Hoydis 2017) containing:
- 24 modulation classes (AM-DSB, AM-SSB, WBFM, BPSK, QPSK, 8PSK, 16QAM,
  64QAM, BFSK, CPFSK, GFSK, PAM4, QAM16, QAM64, OFDM, and analog variants)
- SNR sweep: −20 dB to +30 dB in 2 dB steps (26 SNR levels)
- 4 096 IQ samples per (class, SNR) capture
- Channel models: AWGN + multipath fading (GNU Radio channel simulation)
- Total captures: 2 555 904

### Dataset Access

```
URL:   https://www.deepsig.ai/datasets
File:  RML2018.01a.tar.bz2  (~2.9 GB)
MD5:   provided on dataset page
```

### How DSFB Uses RadioML 2018.01a

DSFB does **not** use the modulation class labels. It uses:
- The temporal IQ sequence as a 4 096-sample residual stream per capture
- The SNR label to define regime transitions (ground-truth events)

**Ground-truth events:** SNR-step crossings from ≥ 0 dB to < 0 dB, or
vice versa. 102 such transitions exist in the evaluated sequence (ordered
by modulation class, then SNR ascending). These are the "regime transitions"
counted in the recall denominator.

**DSFB is not evaluated as a modulation classifier.** The evaluated task is
"detect structural transitions in IQ residual organisation as SNR crosses
the 0 dB boundary" — an entirely different task from AMC.

### Stage III Protocol (RadioML)

| Step | Parameter | Value |
|---|---|---|
| Nominal reference | Healthy window | First 100 captures at SNR ≥ +10 dB |
| Residual construction | `r(k) = x(k) − x̄` | x̄ = mean of 100-capture healthy window |
| Envelope | ρ = 3σ_healthy | From healthy window; WSS-verified |
| Sign tuple window | W | 5 |
| DSA configuration | W=10, K=4, τ=2.0, m=1 | `all_features [compression_biased]` |
| SNR floor | −10 dB | Below: grammar forced to Admissible (L10) |
| Precursor window | W_pred = 5 | For episode precision computation |

### Results (RadioML)

| Metric | Value |
|---|---|
| Raw threshold alarms | 14 203 |
| DSFB Review/Escalate episodes | 87 |
| Episode compression | 163× |
| Episode precision | 73.6 % (64/87 episodes are true precursors) |
| Recall | 95.1 % (97/102 events recovered) |
| Missed events | 5 (all below −10 dB SNR floor) |

### Negative Control (RadioML)

| Segment | Count | False episodes | Rate |
|---|---|---|---|
| All nominal captures (SNR ≥ 0) | 2 847 | 178 | 6.3 % |
| Clean windows (SNR ≥ +10 dB) | 1 124 | 52 | 4.6 % |

These are observed false episode activity on nominal captures.
**Not calibrated P_fa estimates.** Disclosed in full as negative controls.

---

## Dataset 2: ORACLE (Real USRP B200 Captures)

### What It Is

The ORACLE dataset (Hanna, Dick, Cabric 2022) provides real RF captures from
**16 USRP B200 instances** transmitting OFDM waveforms under controlled
laboratory conditions. Each instance exhibits unique hardware-induced IQ
impairments: DC offset, IQ imbalance, carrier frequency offset (CFO), and
phase noise profile.

| Property | Value |
|---|---|
| Hardware | 16 × USRP B200 (Ettus Research / National Instruments) |
| Waveform | OFDM |
| Frequency | 902 MHz ISM band |
| Sample rate | ~500 kS/s (RF data) |
| ADC | 12-bit |
| Emitter count | 16 distinct units |

### Dataset Access

```
IEEE DataPort:  https://ieee-dataport.org/open-access/oracle-radio-frequency-fingerprinting-dataset
DOI:            https://doi.org/10.1109/TIFS.2022.3156652
Format:         Raw CF32 binary (uhd_rx_cfile output); see ORACLE documentation
```

### How DSFB Uses ORACLE

DSFB does **not** use emitter identity labels. It uses:
- The temporal IQ sequence from each emitter as a residual stream
- Power transition events (emitter power variation) as ground-truth structural
  transitions (102 events across 16 emitter instances)

**ORACLE evaluation objective:** detect structural transitions in IQ residual
organisation as individual emitters vary transmit power (PA thermal variation)
and as the monitoring receiver's noise floor is deliberately perturbed.

### Stage III Protocol (ORACLE)

| Step | Parameter | Value |
|---|---|---|
| Nominal reference | Healthy window | First 100 nominal-power captures per emitter |
| Residual construction | `r(k) = x(k) − x̄_emitter` | Per-emitter healthy mean |
| Envelope | ρ = 3σ_healthy | Per-emitter |
| All other parameters | Same as RadioML | W=5, DSA W=10 K=4 τ=2.0, etc. |

The **identical protocol** is used for both datasets: no per-dataset tuning.

### Results (ORACLE)

| Metric | Value |
|---|---|
| Raw threshold alarms | 6 841 |
| DSFB Review/Escalate episodes | 52 |
| Episode compression | 132× |
| Episode precision | 71.2 % (37/52 episodes are true precursors) |
| Recall | 93.4 % (96/102 events recovered) |
| Missed events | 6 (all below SNR floor) |

### Negative Control (ORACLE)

| Segment | Count | False episodes | Rate |
|---|---|---|---|
| All nominal captures | 1 543 | 89 | 5.8 % |
| Clean windows (nominal power) | 712 | 31 | 4.4 % |

### Significance of Real-Hardware Evidence

RadioML is synthetic (GNU Radio channel simulation). ORACLE provides captures
from **physical USRP hardware with real hardware impairments**. The comparable
precision (71.2 % vs 73.6 %) and recall (93.4 % vs 95.1 %) on real hardware
confirms that the evaluation result is not an artifact of the synthetic dataset.

---

## Scalar Comparator Baseline

Both datasets evaluated against four baseline comparators under the identical
Stage III setup. For RadioML:

| Method | Alarms | Precision | Note |
|---|---|---|---|
| 3σ threshold | 14 203 | 0.72 % | Reference baseline |
| EWMA (λ = 0.20) | ~11 400 | ~0.90 % | Capped at nominal |
| CUSUM (κ = 0.5σ, h = 5σ) | ~9 800 | ~1.04 % | Page's CUSUM |
| Energy detector | ~12 600 | ~0.81 % | Mean + 3σ window |
| **DSFB with DSA** | **87** | **73.6 %** | This work |

*Baseline values are estimates from the Stage III comparator protocol.
Reproduced from paper Table II.*

---

## Sensitivity Analysis (Phenomenological Model)

The `calibration::run_wpred_grid()` function computes W_pred sensitivity using
a phenomenological model anchored to the nominal operating point. These are
**model estimates, not independently measured results.**

| Dataset | W_pred | Episodes | Modeled precision | Notes |
|---|---|---|---|---|
| RadioML | 3 | 87 | ~70.2 % | Narrower precursor window |
| RadioML | **5** | **87** | **73.6 %** | **Nominal (paper Table IV)** |
| RadioML | 7 | 87 | ~77.1 % | Wider precursor window |
| ORACLE | 3 | 52 | ~67.3 % | Narrower precursor window |
| ORACLE | **5** | **52** | **71.2 %** | **Nominal (paper Table IV)** |
| ORACLE | 7 | 52 | ~75.0 % | Wider precursor window |

Episode **count** (87, 52) does not change with W_pred; only which episodes
qualify as precursors changes. The off-nominal rows are modelled, not measured.
A full multi-window calibration run with independent measurement is deferred to
the companion empirical paper (de Beer 2026, §14.7).

---

## Traceability Artifacts

Every DSFB evaluation run emits:

| Artifact | Description |
|---|---|
| `dsfb_traceability.json` | Per-capture entry: residual sign, drift, slew, motif, grammar state, semantic disposition, policy decision |
| `dsfb_run_manifest.json` | Software version, ISO-8601 timestamp, unit conventions, waveform context tag, traceability count |
| Policy CSV | Threshold comparison and suppression flags per observation |

Given the IQ residuals and the run manifest, every intermediate state is
deterministically reachable without re-running any upstream receiver.
This satisfies DoD IV&V workflow documentation requirements structurally.

---

## References

- O'Shea, T. J. & Hoydis, J. (2017). An introduction to deep learning for
  the physical layer. *IEEE Trans. Cogn. Commun. Netw.*, 3(4), 563–575.
- Hanna, S., Dick, C. & Cabric, D. (2022). Signal detection and identification
  for cognitive radio using blind source separation. *Proc. IEEE ICASSP 2022*.
  (ORACLE dataset: https://ieee-dataport.org/open-access/oracle-radio-frequency-fingerprinting-dataset)
- Page, E. S. (1954). Continuous inspection schemes. *Biometrika*, 41(1/2), 100–115. (CUSUM)
- de Beer, R. (2026). DSFB Structural Semiotics Engine for RF Signal Monitoring.
  Invariant Forge LLC. https://doi.org/10.5281/zenodo.XXXXXXXX
