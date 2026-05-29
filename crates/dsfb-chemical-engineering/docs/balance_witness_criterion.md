# The balance-witness applicability criterion

> **Prior-art disclosure.** This document states, as prior art in its own right, a falsifiable,
> physics-grounded criterion for *when a mass/energy-balance witness is applicable* to a chemical /
> process-monitoring fault, together with the positive and negative evidence that tests it. It is part
> of the DSFB-Chemical-Engineering disclosure; nothing here is a performance or root-cause claim.

## 1. What a balance witness is

A **balance witness** recomputes a documented conservation closure from the raw instrument columns of a
control volume and admits the closure *residual* to the DSFB grammar as a `ProcessStructure` detector
stream. It keys on the **sustained shift** of the closure above a calibrated baseline band — **not** on
zero-closure, because a real instrumented balance carries a structural offset (see §1.1).

The witnesses are implemented in `crates/dsfb-chemical-engineering-edge/src/balance.rs`
(`balance_residual`, dispatching on `balance.type`):

| `balance.type` | Conserved quantity | Closure residual (per step) |
|---|---|---|
| `mass_three_tank` | tank-2 volume (mass) | `area·dL₂/dt − [c₁₂·sgn(Δh)√|Δh| − c₂₃·sgn(Δh)√|Δh|]` (Torricelli inter-tank flow) |
| `mass_quad_tank` | tank-1 volume (mass) | analogous Johansson quadruple-tank closure |
| `energy_cstr` | reactor enthalpy | `ρcpV·dT/dt − [Q_rxn(Arrhenius) − Q_feed − Q_coolant]` |
| `energy_csth` | heater enthalpy | `ρcpV·dT/dt − [Q_steam − Q_cold-in − Q_loss]` |
| `mass_tank_volume` | tank volume (mass) | `area·dL/dt − factor·(Σ inflows − Σ outflows)`; `outflows` optional (inflow-only regime for unmetered demand) |

`residual[0] = 0` (the closure needs a one-step difference). Units are the engineering units of the
balance. The computation is deterministic and a pure function of the matrix + the roles sidecar.

### 1.1 Closure magnitude is tiered — and the differentiation-noise floor bounds applicability

The two balance families do **not** close to the same baseline magnitude, and the disclosure is honest
about which is which:

- **Mass / volume balances** (`mass_three_tank`, `mass_quad_tank`, `mass_tank_volume`) close to a **small**
  baseline residual when fully metered, because the level *integrates* the metered net flow (no
  derivative-gain amplification). The **linear** `mass_tank_volume` worked example in §5 (`area = Δt =
  factor = 1`) closes to **≈ 0 exactly**; the *nonlinear* Torricelli demonstrators (three-/quadruple-tank)
  close to a **small model-reconstruction offset** — e.g. the three-tank baseline mean `|residual| ≈ 3.4`
  (the Torricelli inter-tank-flow reconstruction error), still ~9× below the leak break (30.6). So "≈ 0"
  is exact only for the linear case; for the Torricelli cases it is "small relative to the fault break".
- **Energy balances** (`energy_cstr`, `energy_csth`) carry a **large structural offset**, not ≈ 0, from
  **two** stacked sources — and the honest budget names both:
  1. a **differentiation-noise floor**: the closure differentiates a measured temperature, `ρc_pV·dT/dt`,
     and the gain `ρc_pV/Δt` (≈ 23 900 J/K ÷ 0.5 min for the CSTR) amplifies per-step thermocouple noise
     `σ_T` (≈ 0.1 K) into a band of order **`ρc_pV·σ_T/Δt ≈ 5×10³ J/min`** (thousands);
  2. the **dominant** term — the **model-form / discretization residual** of the reconstructed
     reaction−cooling difference (an Euler-step + Arrhenius-nonlinearity reconstruction evaluated at the
     *reported* temperature), of order **`~2.5×10⁵ J/min`**, which is what sets the actual baseline band.
  The witness calibrates its band on the baseline and keys on the *sustained relative shift* above it, so
  the headline "7.5×/251×" energy-break ratios are ratios over this **noisy, model-error-dominated**
  baseline, **not** over a clean zero.

This gives the criterion a **quantitative applicability bound**, not only the qualitative gate of §2: an
energy-balance witness can resolve a fault **only if** the fault's sustained closure shift exceeds the
**full calibrated baseline band** = the differentiation-noise floor `ρc_pV·σ_T/Δt` (a valid *lower* bound,
thousands of J/min) **plus** the model-form residual that dominates it (`~10⁵ J/min` for the CSTR). To
lower the band: smooth/centre the derivative or lengthen `Δt` (cuts the noise floor) and improve the
reconstruction model (cuts the dominant model-form term). A slow drift below this band is, correctly,
**not** resolvable — which is why the CSTR thermocouple-drift fault is detected only once the drift has
accumulated past the band (a large, honestly-reported detection delay), whereas the fast insulation-loss
and leak faults clear it within a few samples. This is **clause (iii)** of the criterion (§2): the witness
is applicable only where the reconstruction model is faithful enough that its baseline offset sits below
the fault break.

### 1.2 Estimating the baseline band for a *new* plant (a commissioning procedure)

§1.1 names the two band terms for the studied CSTR; the values are model-specific, so a deploying engineer must
estimate them for their own volume **before** trusting (or excluding) a balance witness. This is a prescriptive,
data-light procedure — it needs only a normal-operation window plus quantities already on the P&ID / instrument
datasheets, so it can be done at commissioning without injecting any fault:

1. **Differentiation-noise floor (a closed-form *lower* bound).** Take the gain on the differentiated term —
   for an energy balance, `G = ρ·c_p·V/Δt` (heat capacity of the metered inventory ÷ the sample period); for a
   level/volume balance with `area·dL/dt`, `G = area/Δt`. Read the per-step sensor noise `σ` of the differentiated
   transmitter from its datasheet repeatability or, better, from the standard deviation of that transmitter over a
   *steady* baseline window. The floor is then `band_diff ≈ G·σ`. (CSTR worked value: `23 900 J/K ÷ 0.5 min ×
   0.1 K ≈ 5×10³ J/min`.) This term is **purely instrumental** — no process model is needed — which is why it is a
   trustworthy lower bound even before the reconstruction model is validated.
2. **Model-form residual (measured directly from baseline closure).** Run the witness's closure reconstruction over
   a **fault-free** window and record the residual stream `e_k` (DSFB already computes and seals this — it is the
   `balance_witness.csv` / `balance_residual` lane). The model-form term is just the **baseline central tendency of
   `|e_k|`** with the noise floor removed in quadrature: `band_model ≈ sqrt( mean(e_k²) − band_diff² )`. No fault is
   needed — a normal run *is* the estimator, because in baseline the only things producing a non-zero residual are
   the reconstruction error plus instrument noise. (CSTR: this is the term that lands at `~2.5×10⁵ J/min` and
   dominates the floor by ~50×.)
3. **The applicability test.** The full calibrated band is `band ≈ band_diff + band_model` (DSFB calibrates exactly
   this on the baseline window). The witness can resolve a candidate fault **iff** the fault's *sustained* closure
   shift Δ exceeds `band`. Compare against the smallest fault you must catch: take its expected magnitude from the
   HAZOP deviation table (e.g. "loss of cooling → ΔQ of order *X* J/min"; "10 % feed-meter bias → Δ of order *Y*").
   If `X, Y < band`, the witness is **out of scope for that fault** — declare it `Catalogued`/blind rather than ship
   a witness that will sit silent, and fall back to the chemometric detectors. If `X, Y ≫ band`, the witness is a
   high-selectivity, physics-grounded sentinel for it.
4. **If the band is too high to be useful, lower it before deploying** (§1.1): smooth/centre the derivative or
   lengthen `Δt` to cut `band_diff`; improve the reconstruction model (better kinetics, a measured `UA`, finer
   discretisation) to cut the dominant `band_model`. Re-estimate from a fresh baseline window and re-test step 3.

The point is that the bound is **estimable from a normal run plus datasheet/HAZOP numbers** — the operator never has
to break their plant to learn whether a balance witness will work on it, and the `Executed`-sentinel-vs-`Catalogued`/
blind decision is made on disclosed, reproducible arithmetic rather than hope.

## 2. The criterion (closure gate)

> A balance witness over a control volume can admit evidence **if and only if**
> **(i)** the volume is **closed and fully metered** for the conserved quantity — *every* cross-boundary
> flow of mass or energy is instrumented — **and**
> **(ii)** the fault makes that **conserved** quantity **appear** non-conserved at the meters: a
> **sensor spoof or drift** corrupting a balance term, or a **leak / sink crossing the metered
> boundary** — **and**
> **(iii)** the closure's **reconstruction model is faithful** enough that its calibrated baseline band
> (the differentiation-noise floor *plus* the model-form residual, §1.1) sits **below** the fault's
> sustained shift, so the break is resolvable above the band.
>
> The witness is structurally **blind** to faults that *respect* conservation inside a fully-metered
> volume (composition or efficiency shifts), to imbalances that an **unmetered** cross-boundary flow
> can silently absorb, and to shifts **smaller than its baseline band** (clause iii — e.g. a drift
> slower than the differentiation-noise floor can resolve).

The criterion is **doubly testable**: it predicts both where the witness fires *and where it must stay
silent*. That is what makes it falsifiable rather than a generic anomaly heuristic.

## 3. Positive evidence — fires, with physical selectivity (real data)

Computed locally on the real, **non-redistributed** testbed data (only the recipe + roles sidecars are
committed; the bytes are gitignored under an iTrust/benchmark agreement):

- **BATADAL** C-Town water network, tank **T1** volume balance (`mass_tank_volume`, inflow-only regime,
  outflow = unmetered district demand). Fires on **exactly** the two labelled attacks that manipulate
  T1's inflow pump **PU2** (peak 1.8× / 1.7× threshold — the spoofed pump flow reports steady inflow
  while the tank drains), stays **quiet** on the three attacks targeting other tanks (0.5–0.9×), and
  false-fires on **0.1 %** of normal-year samples (5 / 3958). Spatial selectivity from conservation
  physics, not a generic flag.
- **SWaT** secure-water-treatment testbed, stage-1 raw-water tank **T101** (`mass_tank_volume`, both
  flow legs metered: `area·dLIT101/dt = FIT101 − FIT201`). Fires on the documented stage-1 `LIT101`
  sensor-spoof attacks (level frozen at 700 mm while the meters show the tank draining → 11–32×
  threshold) and false-fires on **4.4 %** of normal-run blocks (119 / 2700).

  **Scope-stratified recall** (`scripts/swat_scope_recall.py`, classified against the official iTrust
  2015 attack list — the list itself is *not* redistributed). Of the 36 physical-impact attacks, exactly
  **5 touch a T101 balance term** (`LIT101`/`FIT101`/`FIT201`: attacks #3, 21, 30, 33, 36) — these are the
  attacks the closure criterion says the witness *can* see. **Within-scope recall = 5 / 5 = 100 %.** Of the
  30 out-of-scope windows (other stages/points, which the criterion says the witness must be blind to),
  **73 % stay correctly quiet**. So the apparent "13 of 35 fire" is not a miss rate: it is the criterion
  confirmed — full recall on the in-scope balance-term spoofs, structural blindness to the rest.

Four **synthetic** instrumented demonstrators (three-tank, quadruple-tank, CSTR, CSTH) establish the
mechanism under fully documented physics: unmeasured-leak and thermocouple-drift / insulation-loss
faults break the closure (9×–250×) and the witness fires at or within a few samples of the labelled onset.

## 4. Negative evidence — correctly stays silent (rejected by the gate)

Reporting where the method is **structurally inapplicable**, with the conservation reason, is what makes
the positive results interpretable:

| Dataset | Why out of scope | Clause |
|---|---|---|
| Tennessee Eastman | Molar/composition disturbance; no closeable bulk-mass balance | (i) |
| PRONTO multiphase facility | Diverted flow recirculates *within* the metered boundary | (ii) |
| UCI Water Treatment Plant | Consumption incompletely metered; unmetered demand swamps closure | (i) |
| BattLeDIM water network | Incomplete consumer metering (same as UCI WTP) | (i) |
| ASHRAE RP-1043 chiller | Refrigerant-leak fault *conserved* by the first-law water-side energy balance (closure −4.1 vs −3.7 tons, normal vs leak) | (ii) |
| HAI ICS testbed | Tank outflow unmetered → imbalance absorbed | (i) |

## 5. Worked example (synthetic, reproducible)

`crates/.../scripts/gen_historian_fixture.py` emits a fully-synthetic level-controlled tank
(`area = dt = factor = 1`), so the closure reduces to `residual[k] = (LIT[k] − LIT[k−1]) − (FIT_IN −
FIT_OUT)`. In the normal region the level integrates the metered net flow exactly → closure ≈ 0; in the
fault region the level sensor is **spoofed** (frozen) while the meters show net outflow → a sustained
closure break equal to the net-outflow magnitude. The `historian` command emits a balance witness
(`balance_witness.csv` + baseline-vs-tail closure) alongside the Chemical Court Record. The unit tests
in `balance.rs` verify a balanced tank closes to ~0 and a leaking tank breaks the closure to a sustained
−1.

## 6. Why this is the transferable contribution

The DSFB residual grammar is composed of prior-art primitives (moving average, finite difference,
k-of-n quorum, Shannon entropy). The **closure-gate criterion** is the genuinely new, domain-portable
result: a witness that *declares its own domain of validity* and produces **silence, not a false claim,**
outside it. It is disclosed here, in the paper (`\section{Balance-witness applicability criterion}`,
`sec:balancecriterion`), and in the code so that it stands as prior art independent of any specific
deployment.
