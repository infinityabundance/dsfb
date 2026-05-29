# Forensic incident walkthrough (fictional, synthetic) — read in 3 minutes

> **Everything here is invented.** There is **no real company, plant, site, person, or agency**. The
> scenario exists only to show, end to end, what an operator hands the `historian` command and what
> Chemical Court Record comes back, on openly-shareable **synthetic** data (NOT plant data). It makes no
> detection-superiority or root-cause claim.

## The scenario

**Northgate Specialty Chemicals** (fictional), Plant 2, runs exothermic reactor **R-101**, cooled by a
jacket fed from **cooling-water surge tank TK-110**. TK-110 is on level control with three instruments:

| Tag | Meaning | Role |
|---|---|---|
| `LIT-110` | surge-tank level (mm) | controlled variable (carries setpoint / valve output / controller mode) |
| `FIT-110A` | make-up cooling-water inflow (m³/h) | metered inflow |
| `FIT-110B` | draw to the R-101 jacket (m³/h) | metered outflow |

The documented volume balance is `area·dLIT-110/dt = FIT-110A − FIT-110B` (area = dt = factor = 1), so
under normal control the level integrates the metered net flow and the closure residual is ≈ 0.

**The incident.** Mid-batch (sample 54 of 90, `08:54`), `LIT-110` is **spoofed** — the transmitter
freezes — while the meters keep showing a sustained net draw to the jacket. The tank is really draining
(cooling-water inventory is being lost ahead of a reactor temperature excursion), but the level reads
flat. The operator drops the level loop to manual at onset (`controller_mode` auto → manual).

## What the operator provides

Two files (both committed, fully synthetic — produced by `scripts/gen_incident_fixture.py`):

- `data/historian/northgate_r101_incident.csv` — a generic long-format historian export
  (`timestamp,tag,value,unit,quality,phase_id,controller_mode,setpoint,manipulated_variable`),
  90 one-minute timestamps × 3 tags (plus one deliberately `bad`-quality sample to exercise the gate).
- `data/historian/northgate_r101_incident.roles.json` — the roles sidecar declaring variable roles,
  units, and the `mass_tank_volume` balance equation, so a witness can recompute the closure.

## The one command

```sh
cargo run --release -p dsfb-chemical-engineering-edge -- \
  historian crates/dsfb-chemical-engineering-edge/data/historian/northgate_r101_incident.csv
```

`replay_deterministic = true`: every value below is byte-reproducible. The run writes the full Chemical
Court Record bundle to `output-dsfb-chemical-engineering-historian/northgate_r101_incident/`.

## What comes back — the Chemical Court Record

**1. The physics balance witness caught it.** The witness recomputes `dLIT-110 − (FIT-110A − FIT-110B)`
from the raw tags. Closure residual: **0.000 in the baseline → 8.000 per step in the fault region** (the
spoofed level can't fall, so the residual equals the net draw, 34 − 26 = 8 m³/h). NAMUR **NE 107** status
is **`OK` for the 54 pre-onset samples and `Failure` for all 36 post-onset samples** — it flips exactly
at the spoof onset.

**2. The statistical detector bank did *not* reach quorum — and that is recorded, not hidden.** The
generic detectors flagged a candidate interval `[54..89]` but only **one** detector family fired, below
the two-family quorum. Rather than force an alarm, the court logged it as a **rejected candidate**:

| field | value |
|---|---|
| interval | `[54..89]` (36 steps) |
| `rejection_reason` | `QUORUM_NOT_MET` |
| `raw_reason` | `insufficient_families` (1 of 2 required) |
| `missing_context` | "needs an additional co-firing detector family or a longer sustained run" |
| `evidence_hash` | `4ab4f63a…` |

This is the doctrine in one screen: **the physics witness is what catches a balance-term sensor spoof
(exactly as the balance-witness applicability criterion predicts), while the generic statistical bank
honestly abstains — and the abstention is itself a hash-sealed record, not silence.**

**3. The bundle.** `dsfb_chemical_engineering_casefile_v1/` (the canonical, versioned Court Record):

| | |
|---|---|
| admitted episodes | 0 |
| rejected candidates | 1 (logged, with reason + evidence hash) |
| global badges | `ROOT_CAUSE_NOT_ADMITTED`, `REPLAY_VERIFIED` |
| `evidence_root` | `24b590e0c5bc24d60a453a7b551a88bfd8b62d0b93f95d78b02ab9d66ac19d43` |
| `bundle_root` | `f28e9a6c6b2e99d1764f13ae5ccb5ccecebd49bb39bdddc7d0ff8e6b2425d6d8` |
| `replay_deterministic` | true |
| key statement | "It does not emit an alarm. It emits a court record of why an alarm-like structure was or was not admitted." |

## The 3-minute read

1. **Open `operator_report.html`** — the human view: what happened, when (NE 107 flips to `Failure` at
   `08:54`), which witness fired, and the bounded claim.
2. **Confirm the physics** — `balance_witness.csv` shows the closure stepping 0 → 8 at the onset.
3. **See the recorded abstention** — `rejected_candidates.csv` shows the statistical candidate the court
   declined, with the reason and an evidence hash.
4. **Verify it replays** — re-run the command; `evidence_root` and `bundle_root` are byte-identical.

## What this walkthrough does *not* claim

- **Not** a root-cause diagnosis: the bundle carries `ROOT_CAUSE_NOT_ADMITTED`. It establishes that the
  *metered mass balance is inconsistent* — a sensor spoof or a real loss — not which, and not why.
- **Not** a detection-superiority claim against any incumbent system.
- **Not** real data, and **not** a real entity. Fictional scenario, synthetic series, no agencies.
- The balance witness applies here **only because** TK-110 is closed and fully metered for the conserved
  volume and the fault corrupts a balance term — see [docs/balance_witness_criterion.md](balance_witness_criterion.md).
