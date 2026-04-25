# Unitree G1 humanoid teleoperation — oracle protocol

Dataset: **Unitree G1 block-stack teleoperation** (`Makolon0321/unitree_g1_block_stack`).

## Provenance

- **Public archive:** HuggingFace dataset [`Makolon0321/unitree_g1_block_stack`](https://huggingface.co/datasets/Makolon0321/unitree_g1_block_stack).
- **Platform:** Unitree Robotics G1, 23-DoF biped + bimanual humanoid.
- **Sensing:** joint encoders, IMU.
- **Sampling rate:** 10 Hz logged.

## Licence and redistribution

- **Licence:** Apache-2.0.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/unitree_g1.csv`](../data/processed/unitree_g1.csv) (3 671 timesteps from 10 episodes).

## Fetch path

1. The preprocessor downloads the parquet shards into `data/unitree_g1/`.
2. Residual = Euclidean norm of 74-dim whole-body observation-state deviation from 20% early-window nominal per timestep.
3. Run `paper-lock unitree_g1`.

## Residual construction

`r(k) = ‖observation.state(k) − nominal‖₂`.

## Cassie substitution rationale

Originally scoped as Cassie (OSU Dynamic Robotics, RSS 2021); the canonical UMich-BipedLab `measurements_v1.mat`/`true_state_v1.mat` recordings are MATLAB-v5 with Simulink Stateflow opaque-wrapped time series that cannot be decoded outside MATLAB. Unitree G1 replaces the row in the same "real bipedal humanoid hardware teleoperation" category, open-licence and parquet-readable.

## DSFB bounded claim

Surfaces humanoid teleoperation residual structure as grammar episodes. No teleop-skill or imitation claim.

## Reproducibility

- `paper-lock unitree_g1`. Smoke-test: `paper-lock unitree_g1 --fixture`.
