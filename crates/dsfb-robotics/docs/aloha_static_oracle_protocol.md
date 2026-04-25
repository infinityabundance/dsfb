# ALOHA static (coffee) bimanual teleoperation — oracle protocol

Dataset: **ALOHA bimanual static coffee** (Zhao, Kumar, Finn 2023; LeRobot `aloha_static_coffee`).

## Provenance

- **Reference:** Zhao T., Kumar V., Finn C., "Learning Fine-Grained Bimanual Manipulation with Low-Cost Hardware," *RSS 2023*. [arXiv:2304.13705](https://arxiv.org/abs/2304.13705).
- **Public archive:** HuggingFace dataset [`lerobot/aloha_static_coffee`](https://huggingface.co/datasets/lerobot/aloha_static_coffee).
- **Platform:** Real ALOHA bimanual: 2 × 6-DoF ViperX + 2 × 1-DoF grippers (14 DoFs).
- **Sampling rate:** 50 Hz.

## Licence and redistribution

- **Licence:** Apache-2.0.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/aloha_static.csv`](../data/processed/aloha_static.csv) (55 000 frames across 50 episodes).

## Fetch path

1. The preprocessor downloads parquet shards into `data/aloha_static/`.
2. Residual = Euclidean norm of 14-DoF `observation.state` deviation from 20% early-window nominal per timestep.
3. Run `paper-lock aloha_static`.

## Franka-Kitchen substitution rationale

Originally scoped as Franka Kitchen (Gupta et al., CoRL 2019); but `FrankaKitchen-v1` is a Gymnasium MuJoCo simulation, disqualifying it under the crate's real-world-only policy. ALOHA static coffee replaces the row in the same teleop-demonstration regime, on genuinely physical hardware.

## DSFB bounded claim

Surfaces fine-bimanual coffee-pour residual structure as grammar episodes (reach / pour / place phases). No imitation policy claim.

## Reproducibility

- `paper-lock aloha_static`. Smoke-test: `paper-lock aloha_static --fixture`.
