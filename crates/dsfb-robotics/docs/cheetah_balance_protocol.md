# MIT Cheetah 3 / Mini-Cheetah balancing — oracle protocol

Dataset: **MIT Cheetah 3 / Mini-Cheetah open logs** (Katz–Di Carlo–Kim
2019; Bledt et al. 2018).

## Provenance

- **Reference 1:** Katz B., Di Carlo J., Kim S., "Mini Cheetah: A
  Platform for Pushing the Limits of Dynamic Quadruped Control,"
  *IEEE International Conference on Robotics and Automation* (ICRA),
  2019. DOI: [10.1109/ICRA.2019.8794449](https://doi.org/10.1109/ICRA.2019.8794449).
- **Reference 2:** Bledt G., Powell M. J., Katz B., Di Carlo J.,
  Wensing P. M., Kim S., "MIT Cheetah 3: Design and Control of a
  Robust, Dynamic Quadruped Robot," *IEEE/RSJ International
  Conference on Intelligent Robots and Systems* (IROS), 2018.
- **Platform:** Mini-Cheetah / Cheetah 3 quadruped robots from the
  MIT Biomimetic Robotics Lab. Open-source hardware and control
  software.
- **Upstream software / logs:**
  <https://github.com/mit-biomimetics/Cheetah-Software>.
- **Sampling rates:** 1 kHz joint-level control loop, 500 Hz state
  estimator and whole-body MPC.

## Licence and redistribution

- **Licence:** MIT License (the Cheetah-Software repository). Individual
  capture logs accompanying published papers are covered by the
  licence of the paper's supplementary material — typically
  MIT-compatible, but per-capture verification is required before
  redistribution.
- **This crate ships:** a 6-sample illustrative fixture in
  `src/datasets/cheetah3.rs` representing stance → swing → touchdown
  → stance. Real capture logs are not redistributed; users obtain
  them from the MIT Biomimetics release or reproduce them on their
  own hardware under the MIT License.

## Fetch path

1. Clone the MIT Biomimetics Cheetah-Software repository.
2. Use the provided logging infrastructure to capture IMU,
   joint-encoder, and foot-contact logs during a planned gait cycle
   or push-recovery experiment.
3. Place captured logs under `crates/dsfb-robotics/data/cheetah3/`.
4. Run `paper-lock cheetah3`.

## Residual construction — dual channel

Cheetah exposes two complementary residual channels. DSFB combines
them via [`balancing::combine_channels`](../src/balancing.rs) with
the default [`cheetah3::DEFAULT_COMBINE`](../src/datasets/cheetah3.rs)
strategy (`SumOfSquares`):

- **Contact-force residual `r_F(k) = F_GRF,measured(k) − F_MPC,planned(k)`.**
  The whole-body MPC plans a per-foot ground-reaction force each
  cycle; the measured force differs from the plan by this residual,
  which the MPC rolls into the next horizon and discards. This is
  exactly the "discarded residual" DSFB recovers structure from.

- **Centroidal-momentum observer residual
  `r_ξ(k) = c_CoM,IMU(k) − c_CoM,model(k)`.** The IMU-fused centre-
  of-mass estimate differs from the rigid-body model prediction by
  this residual, which the state estimator fuses and discards.

Combined residual: `‖(r_F(k), r_ξ(k))‖` under the default weighting.

## DSFB bounded claim

Paper §10.9 (Cheetah 3 / Mini-Cheetah) bounded claim: *DSFB identifies
structured residual episodes during locomotion and push-recovery
distinguishable from nominal gait-cycle variation*. DSFB does **not**
claim better balance, more robust recovery, earlier fall detection,
or any control-performance improvement.

## Reproducibility

- Headline numbers: `paper-lock cheetah3` over obtained or reproduced
  logs under the pinned toolchain.
- Smoke-test: `paper-lock cheetah3 --fixture` uses the dual-channel
  6-sample fixture for deterministic CI verification.
