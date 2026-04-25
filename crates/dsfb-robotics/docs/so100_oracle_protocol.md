# SO-ARM100 (SO-100) pick-and-place — oracle protocol

Dataset: **HuggingFace LeRobot SO-ARM100 (SO-100)** pick-and-place corpus.

## Provenance

- **Public archive:** HuggingFace dataset [`lerobot/so100`](https://huggingface.co/datasets/lerobot/so100).
- **Platform:** SO-ARM100 (SO-100) — a low-cost 5-DoF DIY arm built on commodity Dynamixel servos. Open-hardware design.
- **Sampling rate:** ~30 Hz logged.

## Licence and redistribution

- **Licence:** Apache-2.0.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/so100.csv`](../data/processed/so100.csv) (19 631 frames).

## Fetch path

1. The preprocessor downloads the LeRobot parquet into `data/so100/`.
2. Residual = Euclidean norm of joint-state deviation from 20% early-window nominal per timestep.
3. Run `paper-lock so100`.

## Residual construction

`r(k) = ‖q(k) − nominal‖₂` for the 5-DoF arm + gripper joint state.

## DSFB bounded claim

Surfaces commodity-servo pick-and-place residual structure as grammar episodes. The high peak ‖r‖² is consistent with low-cost-servo amplitude characteristics — DSFB structures rather than corrects this. No claim about SO-100 servo accuracy.

## Reproducibility

- `paper-lock so100`. Smoke-test: `paper-lock so100 --fixture`.
