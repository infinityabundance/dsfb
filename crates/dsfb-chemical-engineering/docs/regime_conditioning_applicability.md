# When does regime-conditioning help? — an applicability framework

*Companion to the paper's regime-conditioned-envelope result (Tier 3, "newly demonstrated, bounded").
This doc turns the two observed data points into a **predictive, disclosed** framework so a deploying
engineer can decide — before running — whether regime conditioning will lower their baseline
false-positive rate or leave it unchanged. It narrows nothing: it is additional prior-art disclosure of
the technique's applicability boundary across process classes.*

## 1. What regime-conditioning is

The admissibility envelope `[r_min,r_max]×[δ_min,δ_max]×[σ_min,σ_max]` is normally fit on one baseline
window and applied to the whole run. **Regime-conditioning** instead indexes the envelope by a per-sample
regime / phase / operating-point label, so each sample is judged against the envelope of *its own* regime.
The mechanism is read-only and deterministic (the label is an input, not an inference) and is opt-in
(`regime_conditioned` in `PipelineConfig`; auto-enabled for batch-record historian imports that carry a
phase column). It changes **only** which envelope a sample is compared against — never the grammar, the
fusion, or the sealed evidence format.

## 2. The empirical boundary (the two disclosed data points)

| Dataset | Class | Baseline FP, global | Baseline FP, regime-conditioned | Verdict |
|---|---|---|---|---|
| Penicillin fed-batch | fed-batch fermentation, strong growth-phase non-stationarity | 54.0 % | **39.0 %** (↓ 15 pp) | **helps** |
| Gas-sensor array drift | continuous sensor array, within-baseline heterogeneity | 74.3 % | 76.0 % (≈ flat / slightly worse) | **does not help** |

The paper states the boundary in one line: *"regime-conditioning helps when the regime label is aligned
with the axis of non-stationarity."* This doc makes that operational.

## 3. The predictive framework — by process class

Regime-conditioning lowers the baseline false-positive rate **iff** a *known, per-sample* label partitions
the baseline into segments that are each **more stationary than the pooled baseline**. The decisive
question is not "is the process non-stationary?" but "**does my label explain the non-stationarity?**"

| Process class | Is there a label aligned with the non-stationarity? | Expected effect |
|---|---|---|
| **Fed-batch / batch fermentation, polymerisation, crystallisation** (clear lag → growth → stationary → decline phases, each with its own admissible residual band) | **Yes** — the batch phase / recipe step is recorded and *is* the non-stationarity axis | **Helps** (penicillin class). Condition on phase. |
| **Grade-transition / campaign continuous plants** (distinct, labelled product grades or feed campaigns) | **Yes** — the grade/campaign tag partitions operation into quasi-stationary regimes | **Helps.** Condition on grade/campaign. |
| **Start-up / shutdown / load-following** with a recorded mode or load setpoint | **Yes** — the mode/load label tracks the operating point | **Helps** where the label is logged at sample rate. |
| **Steady continuous process with slow drift** (catalyst ageing, fouling) and *no* regime label | **No** label to condition on | **Unknown / no help** — the drift is the thing you are trying to detect, not a baseline regime; conditioning on a coarse label cannot capture it. Use the drift detectors, not envelope conditioning. |
| **Heterogeneous sensor arrays with within-baseline variability** *not* captured by the available label (e.g. per-channel sensor-to-sensor spread that a single batch/run tag does not separate) | **No** — the label exists but is *misaligned* with the variability axis | **Does not help** (gas-sensor class). Conditioning on a label that does not partition the heterogeneity adds bins without adding stationarity, and can slightly worsen FP by thinning each bin's baseline. |

**Rule of thumb:** condition only on a label `L` for which the **within-`L` baseline residual variance is
materially smaller than the pooled baseline variance**. If `L` does not shrink the variance, it will not
shrink the false-positive rate.

## 4. The applicability test (run it on a baseline window, no fault needed)

1. Pick the candidate label `L` (phase, grade, mode, …) recorded at sample rate.
2. On a **fault-free** window, compute the residual triple `(r,δ,σ)` per channel.
3. Compare `Var(residual | pooled baseline)` against the variance-weighted mean of `Var(residual | L = each value)`.
4. **Decide:** if the within-`L` variance is materially lower (a meaningful fraction below pooled), regime
   conditioning is expected to lower baseline FP — enable it. If it is comparable or higher, the label is
   misaligned — **leave conditioning off and report the global envelope** (do not bin for the sake of it).

This is the same discipline as the balance-witness commissioning test (`balance_witness_criterion.md` §1.2):
the decision is made on a normal run with disclosed arithmetic, not by trial-and-error on faults.

## 5. Why this is disclosed as prior art, not narrowed away

The honest negative (gas-sensor 74 %→76 %) is part of the contribution: it delimits exactly where the
technique applies and prevents a downstream actor from patenting "regime-conditioned residual envelopes for
*process X*" as if it were universally beneficial. The framework above claims the applicability boundary
itself — across **all** the process classes named — as deliberately broad prior art, while telling the
operator truthfully when to switch the feature on. It adds a decision procedure; it removes no scope.
