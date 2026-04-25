# DLR Rollin' Justin / LWR-III kinematics — oracle protocol

Dataset: **DLR Rollin' Justin / Light-Weight Robot III joint torque
identification** (DLR Institute of Robotics and Mechatronics).

## Provenance

- **Reference:** Albu-Schäffer A., Hirzinger G. et al. publications
  on the DLR LWR-III and Rollin' Justin platforms. Selected
  excitation trajectories accompanying DLR technical reports and
  follow-up identification papers in *IEEE Robotics and Automation*,
  *IROS*, *ICRA* 2002–2015.
- **Platform:** DLR LWR-III seven-DoF research arm and the mobile
  two-arm humanoid Rollin' Justin. Both platforms carry **direct
  link-side torque sensing** — the signature DLR instrumentation
  choice that complements the motor-side reconstruction used by
  Panda and UR10.
- **Sampling rate:** 1 kHz.

## Licence and redistribution — manifest-only

- **Licence:** DLR research artefacts require a **Data Use Agreement**
  with the DLR Institute of Robotics and Mechatronics for full
  trajectory access. Published excerpt tables in the DLR papers may
  be used under fair-use academic quotation.
- **This crate ships:** a 4-sample illustrative fixture in
  `src/datasets/dlr_justin.rs` plus a manifest entry in
  `data/slices/dlr_justin_slice.json` (Phase 9). **No raw sample
  redistribution.** `paper-lock dlr_justin` (real-data mode) exits
  with `EX_USAGE (64)` and this file's URL if the corpus is absent.

## Fetch path (requires DUA)

1. Contact DLR Institute of Robotics and Mechatronics
   (<https://www.dlr.de/rm/>) for a research-access Data Use Agreement.
2. Once access is granted, extract to
   `crates/dsfb-robotics/data/dlr_justin/`.
3. The published identified parameter vector `θ̂_dlr` is cited in
   Albu-Schäffer et al.'s LWR-III papers; the adapter embeds a
   representative vector in Phase 9 for reproducibility.
4. Run `paper-lock dlr_justin`.

## Residual construction

For each timestep:

1. `τ_pred(k) = Y(q(k), q̇(k), q̈(k)) · θ̂_dlr`.
2. DSFB residual: `r(k) = ‖τ_link,measured(k) − τ_pred(k)‖`.

## Why this dataset is load-bearing

DSFB's paper §10.7 uses DLR Justin to demonstrate the framework on a
**heavy-arm, humanoid-class platform with direct link-side torque
sensing**, contrasting with the lighter research arms (KUKA LWR,
Panda) and the industrial cobot (UR10). The platform diversity is
what lets the paper claim cross-vendor, cross-topology, cross-sensing
consistency — not a specific DLR-only property.

## DSFB bounded claim

Same structural-identification posture as KUKA LWR and Panda Gaz.
No fault-detection claim.

## Reproducibility

- Headline numbers require the DUA corpus under `paper-lock dlr_justin`.
- Smoke-test: `paper-lock dlr_justin --fixture` uses the 4-sample
  fixture for deterministic CI verification without the corpus.
