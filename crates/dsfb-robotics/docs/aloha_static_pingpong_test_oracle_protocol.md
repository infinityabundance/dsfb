# ALOHA static ping-pong test — oracle protocol

Dataset: **ALOHA bimanual static ping-pong rhythmic transfer** (LeRobot real bimanual, 2024).

## Provenance

- **Public archive:** HuggingFace dataset [`lerobot/aloha_static_pingpong_test`](https://huggingface.co/datasets/lerobot/aloha_static_pingpong_test).
- **Platform:** Real physical ALOHA bimanual.
- **Sampling rate:** 50 Hz.

## Licence and redistribution

- **Licence:** Apache-2.0.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/aloha_static_pingpong_test.csv`](../data/processed/aloha_static_pingpong_test.csv) (6 000 frames).

## Fetch path

1. The preprocessor downloads the LeRobot parquet into `data/aloha_static_pingpong_test/`.
2. Residual = Euclidean norm of 14-DoF `observation.state` deviation from 20% early-window nominal per timestep.
3. Run `paper-lock aloha_static_pingpong_test`.

## Residual construction

`r(k) = ‖observation.state(k) − nominal‖₂`.

## DSFB bounded claim

Surfaces fast bimanual rhythmic structure as recurrent Boundary episodes. The absence of Violations confirms ALOHA's controller envelope contains the rhythm tightly. No claim of period-detection or fault-detection.

## Reproducibility

- `paper-lock aloha_static_pingpong_test`. Smoke-test: `paper-lock aloha_static_pingpong_test --fixture`.
