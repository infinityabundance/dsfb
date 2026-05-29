# Catalogue-expansion targets — disclosed (catalogued, NOT executed) prior-art surface

> **Status: catalogued / disclosed, not executed.** This document widens the prior-art surface by *naming*
> the next detectors, fault signatures, public benchmarks, and balance-witness types the framework is
> designed to absorb — exactly as the existing atlas marks 14/57 detectors *executed* and the rest
> *catalogued*. Listing a target here is an **enabling disclosure**, not a capability claim: nothing below is
> run end-to-end yet, and each item is honestly marked. Executing any of them is a governed change (it
> re-freezes `atlas_hash_v1` and, for new executed detectors, the pipeline replay surface) handled in a
> dedicated pass. Breadth is the strategy; this disclosure widens it without overclaiming.

## Why a separate disclosure doc
Executing a new detector changes every dataset's grammar/fusion → a full replay + bundle-root re-freeze;
executing a new fault signature needs a faithful demonstrator that DSFB genuinely catches; adding to the
57-detector census ripples the cited count through the paper/figures/README. Those are deliberate governed
changes. *Cataloguing* a target is free and risk-free, and is itself prior art — so we disclose the roadmap
here first, then execute selected targets carefully.

## Public-benchmark targets (catalogued)
| Benchmark | Mechanism class | Cheap residual channels | Why a good executed target | Status |
|---|---|---|---|---|
| **DAMADICS** (Development And Application of Methods for Actuator Diagnosis in Industrial Control Systems) | control-valve actuator faults (stiction, sizing, bias, leakage) | PV / MV(OP) / SP, flow | a labelled actuator-fault benchmark; pairs directly with `ValveStictionWitnessV1` + the control-loop context map | catalogued |
| **CWRU bearing** (Case Western Reserve rolling-element bearing) | bearing inner/outer-race + ball faults | vibration → spectral residuals | exercises the spectral/wavelet grammar extensions (frequency-domain motifs) + multi-physics cross-witnessing | catalogued |
| **TEP extended IDV set** (beyond the 5 executed IDVs) | the remaining Tennessee Eastman disturbances | the 52 TEP channels | broadens the executed simulator-fault coverage on an already-vendored benchmark | catalogued |
| **BattLeDIM / L-Town** | water-network leaks (real SCADA) | tank levels + pump flows | a second open real-data mass-balance target alongside BATADAL | catalogued |

## Detector targets (catalogued — widen the executed bank beyond the current 14/57)
Closest-to-existing-primitives first (lowest execution risk): **Hotelling-T² with EWMA smoothing**,
**multivariate CUSUM (MCUSUM)**, **kernel-PCA SPE** (nonlinear residual), **dynamic-PCA (lagged-augmented)**,
**independent-component SPE**, **Hawkins' robust SPE**, **canonical-variate-analysis residual**. Each is
already named in the 57-detector catalogue; executing it is a governed atlas + replay re-freeze.

## Fault-signature targets (catalogued — the 6 not-yet-executed of F1–F12, + new mechanisms)
The atlas already catalogues F2 (cavitation), F4 (HX bypass), F5 (pump-bearing), F10 (blockage),
F11 (imbalance), F12 (refrigerant) as *not executed*; each needs a faithful synthetic demonstrator (a signal
that exhibits the documented motif in cheap sensors) before its status flips to `Executed`. New candidate
mechanisms to catalogue: **fouling-progression (slow HX/area decay)**, **column flooding/weeping**,
**compressor surge**, **steam-trap failure**, **filter blinding**.

## Balance-witness targets (catalogued — see Wave 3 / P75)
`TankInventoryBalance`, `SplitterMixerBalance`, `HeatExchangerEnergyBalance`, `ReactorStoichiometricBalance`,
`SeparatorComponentBalance`, `UtilityLoopBalance` — deterministic closure witnesses; no root-cause claim.

## Honesty
Everything here is **catalogued, not executed**. The framework's value claim is unchanged: it does not assert
that these targets are detected today; it discloses, as prior art, the residual-semiotics apparatus designed
to absorb them. Executing a target is a separate, governed, tested step with its own re-freeze and a
demonstrator that DSFB genuinely catches — never a status flip without evidence.
