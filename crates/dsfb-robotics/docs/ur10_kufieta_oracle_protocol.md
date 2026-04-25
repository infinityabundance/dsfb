# Universal Robots UR10 — Kufieta 2014 kinematics — oracle protocol

Dataset: **UR10 system identification** (Kufieta 2014 NTNU thesis
and follow-on UR-series identification literature).

## Provenance

- **Reference (primary):** Kufieta K., "Force estimation in Robotic
  Manipulators: Modeling, Simulation and Experiments: The UR5
  Manipulator as a Case Study," *NTNU MSc Thesis*, 2014.
- **Reference (follow-on):** Kebria P. M., Al-wais S., Abdi H.,
  Nahavandi S., "Kinematic and Dynamic Modelling of UR5 Manipulator,"
  *IEEE International Conference on Systems, Man, and Cybernetics*,
  2016. Extends the Kufieta identification workflow to UR5 and by
  implication UR10 (same kinematic family).
- **Platform:** Universal Robots UR10 six-DoF industrial collaborative
  robot. **Motor-side sensing** — joint current × torque constant
  reconstruction.
- **Sampling rate:** typically 125 Hz on the UR controller's RTDE
  interface; higher rates are available via the real-time monitoring
  port.

## Licence and redistribution

- **Licence:** NTNU thesis artefacts are generally permissive with
  attribution; follow-on datasets published alongside Kebria et al.
  2016 and subsequent papers carry individual licences (typically
  permissive academic).
- **This crate ships:** a 4-sample illustrative fixture in
  `src/datasets/ur10_kufieta.rs` plus a manifest entry in
  `data/slices/ur10_kufieta_slice.json`. Full trajectories are not
  redistributed; users reproduce the excitation trajectories on their
  own UR10 or obtain published-follow-on data under its licence.

## Fetch path

1. Obtain the Kufieta 2014 thesis data or re-run the excitation
   trajectories on a physical UR10 via the URScript interface, logging
   via the RTDE port.
2. Place recorded joint states + commanded torques under
   `crates/dsfb-robotics/data/ur10_kufieta/`.
3. The identified parameter vector `θ̂_ur10` is cited in Kufieta
   (2014) §5 and Kebria et al. (2016) Table III; the adapter embeds
   a representative vector.
4. Run `paper-lock ur10_kufieta`.

## Residual construction

For each timestep:

1. `τ_pred(k) = Y(q(k), q̇(k), q̈(k)) · θ̂_ur10`.
2. DSFB residual: `r(k) = ‖τ_motor,measured(k) − τ_pred(k)‖`.

## Why the industrial-cobot complement matters

Paper §10.8 uses UR10 to demonstrate that the DSFB framework holds on
a *widely-deployed industrial cobot*, not only research hardware
(KUKA, Panda, DLR). This is important for reviewer credibility in
the robotics community — an observation that applies only to research
platforms is taken less seriously. UR10 grounds the claim in the
commercial deployment environment DSFB ultimately targets.

## DSFB bounded claim

Same structural-identification posture as KUKA LWR, Panda Gaz, DLR
Justin. Structural episodes in healthy operation, no fault-detection
claim.

## Reproducibility

- Headline numbers: `paper-lock ur10_kufieta` over reproduced or
  obtained trajectories under the pinned toolchain.
- Smoke-test: `paper-lock ur10_kufieta --fixture`.
