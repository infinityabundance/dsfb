# ergoCub Sorrentino balancing-torque-control — oracle protocol

Dataset: **ergoCub Sorrentino balancing experiment** (Sorrentino et al., IIT ami-iit, IEEE RAL 2025).

## Provenance

- **Reference:** Sorrentino I. et al., "Whole-body torque-control for compliant humanoid balancing," *IEEE RAL 2025*.
- **Public archive:** [`ami-iit/paper_sorrentino_2025_ral_balancing`](https://github.com/ami-iit/paper_sorrentino_2025_ral_balancing) (BSD-3-Clause).
- **Platform:** ergoCub humanoid (same hardware family as the §10.9 push-recovery dataset, but a different task and control regime).
- **Sensing:** F/T sensors at feet, joint encoders, IMU.
- **Sampling rate:** 100 Hz logged.

## Licence and redistribution

- **Licence:** BSD-3-Clause (IIT ami-iit).
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/icub3_sorrentino.csv`](../data/processed/icub3_sorrentino.csv) (4 654 timesteps).

## Fetch path

1. Clone `ami-iit/paper_sorrentino_2025_ral_balancing` into `data/icub3_sorrentino/`.
2. The preprocessor reads the HDF5 recording; foot F/T sensor keys are tried in order (`l_foot_ft_sensor` / `l_foot_ft` etc.) for compatibility.
3. Run `paper-lock icub3_sorrentino`.

## Residual construction

`r(k) = ‖[contact-wrench-residual; centroidal-tracking-error]‖₂`.

## DSFB bounded claim

Surfaces sustained whole-body torque-control perturbation structure as grammar episodes. The high Violation fraction in the sample is the structural fingerprint of an intentionally-perturbed balancing trial — not a fault claim.

## Reproducibility

- `paper-lock icub3_sorrentino`. Smoke-test: `paper-lock icub3_sorrentino --fixture`.
