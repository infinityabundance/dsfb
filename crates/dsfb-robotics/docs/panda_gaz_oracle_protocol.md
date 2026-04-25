# Franka Emika Panda — Gaz 2019 kinematics — oracle protocol

Dataset: **Franka Emika Panda dynamic identification** (Gaz,
Cognetti, Oliva, Robuffo Giordano, De Luca 2019).

## Provenance

- **Reference:** Gaz C., Cognetti M., Oliva A., Robuffo Giordano P.,
  De Luca A., "Dynamic identification of the Franka Emika Panda
  robot with retrieval of feasible parameters using penalty-based
  optimization," *IEEE Robotics and Automation Letters*, 4(4),
  2019, 4147–4154.
  DOI: [10.1109/LRA.2019.2931248](https://doi.org/10.1109/LRA.2019.2931248).
- **Platform:** Franka Emika Panda, seven-DoF research manipulator.
- **Sensing side:** **motor-side**. Torque is reconstructed from
  measured joint current via the Panda's published motor constants
  post-transmission (gearbox compliance absorbed into the identified
  model).
- **Sampling rate:** 1 kHz.
- **Trajectory family:** excitation trajectories designed to excite
  the parameter-base-set of the standard rigid-body dynamic model.

## Licence and redistribution

- **Licence:** accompanying-paper artefact, academic research use.
  The 2019 paper provides feasible parameter values and trajectory
  design; raw time series are not formally redistributed under the
  paper's licence.
- **This crate ships:** the Gaz 2019 published feasible parameter
  vector is embedded as a `const` inside the adapter in Phase 9
  (Phase 3 ships a placeholder alongside the 5-sample illustrative
  fixture). Raw time series are not redistributed.

## Fetch path

1. Reproduce the Gaz 2019 excitation trajectories on a Franka Panda
   using the published trajectory parameters, or obtain the raw
   data under arrangement with the authors (Sapienza / Rennes).
2. Place extracted data under `crates/dsfb-robotics/data/panda_gaz/`.
3. Run `paper-lock panda_gaz`.

## Residual construction

For each timestep:

1. `τ_pred(k) = Y(q(k), q̇(k), q̈(k)) · θ̂_panda` using the Gaz 2019
   identified parameter vector.
2. DSFB residual: `r(k) = ‖τ_motor,measured(k) − τ_pred(k)‖` via
   [`kinematics::tau_residual_norm`](../src/kinematics.rs).

## Paired contrast with link-side kinematics

The paper's §10.6 evaluation uses Panda (motor-side) paired with
KUKA LWR and DLR Justin (link-side) to demonstrate that DSFB's
grammar behaviour is consistent across *sensing modalities*. The
pairing is load-bearing: it shows the framework does not depend on
link-side instrumentation and therefore generalises to
industrial-cobot platforms that only measure motor current.

## DSFB bounded claim

Same posture as KUKA LWR: *DSFB identifies structured residual
episodes in healthy operation distinguishable from identification
noise*. No fault-detection claim. See `README.md` §"Non-Claims".

## Reproducibility

- Headline numbers: `paper-lock panda_gaz` over the full trajectory
  set under the pinned toolchain.
- Smoke-test: `paper-lock panda_gaz --fixture`.
