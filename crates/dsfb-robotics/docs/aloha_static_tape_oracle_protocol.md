# ALOHA static tape — oracle protocol

Dataset: **ALOHA bimanual static tape** (LeRobot real bimanual teleoperation, 2024).

## Provenance

- **Public archive:** HuggingFace dataset [`lerobot/aloha_static_tape`](https://huggingface.co/datasets/lerobot/aloha_static_tape).
- **Platform:** Real physical ALOHA bimanual (same hardware as §10.14 coffee corpus).
- **Sampling rate:** 50 Hz.

## Licence and redistribution

- **Licence:** Apache-2.0.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/aloha_static_tape.csv`](../data/processed/aloha_static_tape.csv) (35 000 frames across 50 episodes).

## Fetch path

1. The preprocessor downloads the LeRobot parquet into `data/aloha_static_tape/`.
2. Residual = Euclidean norm of 14-DoF `observation.state` deviation from 20% early-window nominal per timestep.
3. Run `paper-lock aloha_static_tape`.

## Residual construction

`r(k) = ‖observation.state(k) − nominal‖₂`.

## DSFB bounded claim

Surfaces the rest-versus-active duty cycle of tape-attachment as alternating Admissible / Boundary grammar episodes. No imitation claim.

## Reproducibility

- `paper-lock aloha_static_tape`. Smoke-test: `paper-lock aloha_static_tape --fixture`.
