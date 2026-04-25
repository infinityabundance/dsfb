# KUKA LWR kinematics — oracle protocol

Dataset: **KUKA LWR joint torque identification** (Jubien, Gautier,
Janot 2014).

## Provenance

- **Reference:** Jubien A., Gautier M., Janot A., "Dynamic
  identification of the Kuka LWR robot using motor torques and
  joint torque sensors data," *IFAC Proceedings Volumes*, 47(3),
  2014, 8777–8783.
- **Platform:** KUKA LWR IV, seven-DoF research arm with
  **direct link-side torque sensing** on every joint (DLR-derived
  Light-Weight Robot III architecture).
- **Sampling rate:** 1 kHz.
- **Trajectory family:** excitation trajectories designed for
  full-dynamic-parameter identification with the standard Gautier
  approach.

## Licence and redistribution

- **Licence:** academic-fair-use only. The 2014 paper provides
  the dataset as an accompanying research artefact; there is no
  formal redistribution licence. Users who need the full corpus
  should contact the authors or their institutions (IRCCyN Nantes).
- **This crate ships:** a manifest entry plus a 6-sample
  illustrative slice in `src/datasets/kuka_lwr.rs::FIXTURE` for
  smoke testing. Full trajectories are not redistributed.

## Fetch path

1. Obtain the dataset from the authors of Jubien et al. 2014 or a
   research-access mirror under their terms.
2. Place extracted data under `crates/dsfb-robotics/data/kuka_lwr/`.
3. Run `paper-lock kuka_lwr` over the full corpus.

## Residual construction

For each timestep:

1. Compute the predicted torque
   `τ_pred(k) = Y(q(k), q̇(k), q̈(k)) · θ̂_kuka` using the Jubien 2014
   identified parameter vector `θ̂_kuka`. The adapter embeds this
   vector as a `const` so the residual is reproducible without an
   in-tree identification step.
2. DSFB residual norm:
   `r(k) = ‖τ_link,measured(k) − τ_pred(k)‖`
   using [`kinematics::tau_residual_norm`](../src/kinematics.rs).

## DSFB bounded claim

Paper §10.4 (KUKA LWR) bounded claim: *DSFB identifies structured
residual episodes in healthy operation distinguishable from
identification noise*. The dataset contains **no labelled faults**,
so no fault-detection or -classification claim is made. The property
is structural, not empirical-PHM.

## Reproducibility

- Headline numbers: `paper-lock kuka_lwr` over the full Jubien
  trajectory set.
- Smoke-test: `paper-lock kuka_lwr --fixture`.
