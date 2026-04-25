# ANYmal-C GrandTour parkour — oracle protocol

Dataset: **ANYmal-C GrandTour outdoor locomotion** (ETH-Zürich Legged Robotics, 2024).

## Provenance

- **Reference:** Miki T., Lee J., Hwangbo J., Wellhausen L., Koltun V., Hutter M., "Learning robust perceptive locomotion for quadrupedal robots in the wild," *Science Robotics* 7, 62 (2022).
- **Open data:** HuggingFace dataset [`leggedrobotics/grand_tour_dataset`](https://huggingface.co/datasets/leggedrobotics/grand_tour_dataset).
- **Platform:** ANYbotics ANYmal-C, 12-DoF quadruped.
- **Sensing:** joint encoders + IMU + foot contact estimates.
- **Sampling rate:** 400 Hz nominal; preprocessor decimates to per-step scalar.

## Licence and redistribution

- **Licence:** CC-BY-SA-4.0.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/anymal_parkour.csv`](../data/processed/anymal_parkour.csv) (3 231 timesteps).

## Fetch path

1. The preprocessor downloads a slice of `grand_tour_dataset` into `data/anymal_parkour/`.
2. Residual = Euclidean norm of joint-state + IMU deviation from per-episode nominal stance.
3. Run `paper-lock anymal_parkour`.

## Residual construction

`r(k) = ‖[q(k); IMU(k)] − nominal‖₂` aggregated across the recording.

## DSFB bounded claim

Surfaces outdoor-terrain locomotion structure (stance/swing/recovery transitions) as grammar episodes. No fault-detection or robustness claim against the original learning objective.

## Reproducibility

- `paper-lock anymal_parkour`. Smoke-test: `paper-lock anymal_parkour --fixture`.
