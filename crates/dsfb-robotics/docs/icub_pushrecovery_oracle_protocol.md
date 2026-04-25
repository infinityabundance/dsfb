# ergoCub push-recovery — oracle protocol

Dataset: **ergoCub humanoid push-recovery experiment** (Romualdi, Viceconte et al., IIT ami-iit, IEEE Humanoids 2024).

## Provenance

- **Reference:** Romualdi G., Viceconte P., et al., "Online DCM Trajectory Generation for Push Recovery of Torque-Controlled Humanoid Robots," *IEEE Humanoids 2024*.
- **Public data:** [`ami-iit/paper_romualdi_viceconte_pushing_recovery_iros_2024`](https://github.com/ami-iit/paper_romualdi_viceconte_pushing_recovery_iros_2024) (BSD-3-Clause).
- **Platform:** ergoCub humanoid, 23-DoF whole-body.
- **Sensing:** F/T sensors at feet, joint encoders, IMU.
- **Sampling rate:** ~100 Hz.

## Licence and redistribution

- **Licence:** BSD-3-Clause (IIT ami-iit).
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/icub_pushrecovery.csv`](../data/processed/icub_pushrecovery.csv) (13 092 timesteps). Raw recordings are fetched directly from the ami-iit GitHub release.

## Fetch path

1. Clone `ami-iit/paper_romualdi_viceconte_pushing_recovery_iros_2024` into `data/icub_pushrecovery/`.
2. Run `scripts/preprocess_datasets.py` to extract the wrench-residual + centroidal-momentum-residual streams.
3. Run `paper-lock icub_pushrecovery`.

## Residual construction

Per-timestep: `r(k) = sqrt(r_W² + r_ξ²)` where `r_W` is contact-wrench residual and `r_ξ` is centroidal-momentum tracking error.

## DSFB bounded claim

Surfaces the push-perturbation structure (pre-push quiescent / push-event / recovery-transient phases) as grammar transitions that a scalar wrench RMS summary collapses. No earlier-detection or fault-classification claim.

## Reproducibility

- `paper-lock icub_pushrecovery`. Smoke-test: `paper-lock icub_pushrecovery --fixture`.
