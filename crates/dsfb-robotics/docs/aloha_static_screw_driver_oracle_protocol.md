# ALOHA static screw-driver — oracle protocol

Dataset: **ALOHA bimanual static screw-driver tool-use** (LeRobot real bimanual, 2024).

## Provenance

- **Public archive:** HuggingFace dataset [`lerobot/aloha_static_screw_driver`](https://huggingface.co/datasets/lerobot/aloha_static_screw_driver).
- **Platform:** Real physical ALOHA bimanual.
- **Sampling rate:** 50 Hz.

## Licence and redistribution

- **Licence:** Apache-2.0.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/aloha_static_screw_driver.csv`](../data/processed/aloha_static_screw_driver.csv) (20 000 frames).

## Fetch path

1. The preprocessor downloads the LeRobot parquet into `data/aloha_static_screw_driver/`.
2. Residual = Euclidean norm of 14-DoF `observation.state` deviation from 20% early-window nominal per timestep.
3. Run `paper-lock aloha_static_screw_driver`.

## Residual construction

`r(k) = ‖observation.state(k) − nominal‖₂`.

## DSFB bounded claim

Surfaces sustained tool-use activity as recurrent Boundary structure (compression 0.939). The 518 Violations correspond to transient peak motions during torque-application phases. No tool-use skill claim.

## Reproducibility

- `paper-lock aloha_static_screw_driver`. Smoke-test: `paper-lock aloha_static_screw_driver --fixture`.
