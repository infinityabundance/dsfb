# MIT Mini-Cheetah / Cheetah 3 — open balancing logs — oracle protocol

Dataset: **MIT Mini-Cheetah open locomotion logs** (Katz, Di Carlo, Kim 2019; UMich-CURLY redistribution).

## Provenance

- **Reference:** Katz B., Di Carlo J., Kim S., "Mini Cheetah: A platform for pushing the limits of dynamic quadruped control," *IEEE ICRA 2019*, 6295–6301.
  DOI: [10.1109/ICRA.2019.8793865](https://doi.org/10.1109/ICRA.2019.8793865).
- **Open-data redistribution:** UMich-CURLY public log archive (`air_pronking_gait/`).
- **Platform:** Mini-Cheetah, MIT 12-DoF quadruped.
- **Sensing:** joint encoders, IMU, contact force estimates.
- **Sampling rate:** 500 Hz nominal.

## Licence and redistribution

- **Licence:** academic open-data, attribution to Katz–Di Carlo–Kim 2019.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/cheetah3.csv`](../data/processed/cheetah3.csv) (48 972 timesteps). Raw `.mat` files are downloaded from the UMich-CURLY archive into `data/cheetah3/` by the preprocessor.

## Fetch path

1. The preprocessor [`scripts/preprocess_datasets.py`](../scripts/preprocess_datasets.py) downloads the raw `air_pronking_gait/mat/air_jumping_gait.mat` into `data/cheetah3/`.
2. Residual stream is computed as Euclidean norm of (contact-force residual, centroidal-momentum residual) per timestep.
3. Run `paper-lock cheetah3`.

## Residual construction

For each timestep:
1. `r_F(k)` = contact-force-residual (MPC-predicted minus measured GRF).
2. `r_ξ(k)` = centroidal-momentum-residual (CoM tracking error in the centroidal frame).
3. DSFB residual: `r(k) = sqrt(r_F² + r_ξ²)` via [`balancing::combine_channels`](../src/balancing.rs).

## DSFB bounded claim

DSFB identifies structured residual episodes during locomotion (stance/swing/touchdown phases) that conventional MPC-tracking-error monitoring summarises as a single magnitude trajectory. No fault-detection claim.

## Reproducibility

- `paper-lock cheetah3` over the full pronking-gait recording.
- Smoke-test: `paper-lock cheetah3 --fixture`.
