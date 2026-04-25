# Mobile ALOHA wipe-wine — oracle protocol

Dataset: **Mobile ALOHA wipe-wine** (Fu, Zhao, Finn 2024).

## Provenance

- **Reference:** Fu Z., Zhao T., Finn C., "Mobile ALOHA: Learning Bimanual Mobile Manipulation with Low-Cost Whole-Body Teleoperation," 2024. [arXiv:2401.02117](https://arxiv.org/abs/2401.02117).
- **Public archive:** HuggingFace dataset [`lerobot/aloha_mobile_wipe_wine`](https://huggingface.co/datasets/lerobot/aloha_mobile_wipe_wine).
- **Platform:** Mobile ALOHA — bimanual ALOHA arms mounted on a holonomic mobile base. 14 DoFs arm + 4 DoFs base.
- **Sampling rate:** 50 Hz.

## Licence and redistribution

- **Licence:** Apache-2.0.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/mobile_aloha.csv`](../data/processed/mobile_aloha.csv) (65 000 frames).

## Fetch path

1. The preprocessor downloads the LeRobot parquet into `data/mobile_aloha/`.
2. Residual = Euclidean norm of arm + base whole-body state deviation from 20% early-window nominal per timestep.
3. Run `paper-lock mobile_aloha`.

## Residual construction

`r(k) = ‖observation.state(k) − nominal‖₂` over the 18-dim mobile + bimanual state.

## DSFB bounded claim

Surfaces mobile-base + bimanual coordination structure (active-wipe vs base-handoff phases) as grammar episodes. No imitation policy claim.

## Reproducibility

- `paper-lock mobile_aloha`. Smoke-test: `paper-lock mobile_aloha --fixture`.
