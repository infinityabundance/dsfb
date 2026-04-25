# DROID — oracle protocol

Dataset: **DROID Distributed Robot Manipulation** (Khazatsky et al., Stanford/TRI 2024).

## Provenance

- **Reference:** Khazatsky A. et al., "DROID: A Large-Scale In-the-Wild Robot Manipulation Dataset," 2024. [arXiv:2403.12945](https://arxiv.org/abs/2403.12945).
- **Public archive:** Stanford OPEN/DROID (HuggingFace `KarlP/droid_100`).
- **Platform:** Franka Emika Panda, 76 000+ teleoperation demonstrations across 18 institutions.
- **Sensing:** joint encoders + Cartesian end-effector + RGB cameras.
- **Sampling rate:** 15 Hz logged.

## Licence and redistribution

- **Licence:** CC-BY-4.0.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/droid.csv`](../data/processed/droid.csv) (32 212 timesteps from the 100-episode slice). Raw RLDS is downloaded by the preprocessor into `data/droid/`.

## Fetch path

1. The preprocessor downloads `KarlP/droid_100` from HuggingFace into `data/droid/`.
2. Residual = Euclidean norm of 7-DoF joint-state deviation from per-trajectory early-window nominal.
3. Run `paper-lock droid`.

## Residual construction

`r(k) = ‖q(k) − q̂_nominal‖₂` for the 7-DoF Panda joint state, computed by [`scripts/preprocess_datasets.py::preprocess_droid`](../scripts/preprocess_datasets.py).

## DSFB bounded claim

DSFB structures kinematic-imitation-residual into grammar episodes that surface the multi-task / multi-institution variation as recurrent Boundary structure. No imitation-learning policy claim.

## Reproducibility

- `paper-lock droid`. Smoke-test: `paper-lock droid --fixture`.
