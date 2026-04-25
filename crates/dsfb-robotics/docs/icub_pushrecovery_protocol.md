# IIT iCub push-recovery balancing — oracle protocol

Dataset: **IIT iCub whole-body push-recovery logs** (Nori, Traversaro,
Natale et al., IIT iCub Facility).

## Provenance

- **Reference (foundation):** Nori F., Traversaro S., Eljaik J.,
  Romano F., Del Prete A., Pucci D., "iCub whole-body control
  through force regulation on rigid non-coplanar contacts,"
  *Frontiers in Robotics and AI*, 2, 2015.
- **Reference (push-recovery):** Nava G., Romano F., Nori F.,
  Pucci D., "Stability analysis and design of momentum-based
  controllers for humanoid robots," *IEEE/RSJ IROS*, 2016.
- **Reference (iCub-3 update):** Parmiggiani A. et al., "iCub-3:
  the design of the next generation of humanoid robots," iCub Tech
  publications, 2022.
- **Platform:** iCub humanoid robot (full whole-body joint
  instrumentation, 6-axis F/T sensors at wrists/ankles, IMU).
  Full-body joint state, contact wrenches, whole-body centroidal
  momentum.

## Licence and redistribution — manifest-only

- **Licence:** IIT research data is typically released under a
  **Data Use Agreement** with the iCub Tech facility. Published
  figure/table values in the iCub papers may be quoted under
  academic fair use. Full experiment logs require a DUA.
- **This crate ships:** a 6-sample illustrative dual-channel fixture
  in `src/datasets/icub_pushrecovery.rs` representing pre-push →
  push → recovery. Real logs are **not redistributed**.
- `paper-lock icub_pushrecovery` (real-data mode) exits with
  `EX_USAGE (64)` and this file's URL if the corpus is absent.

## Fetch path (requires DUA)

1. Contact IIT iCub Tech (<https://icub-tech-iit.github.io/>) for a
   research-access Data Use Agreement covering push-recovery log
   corpora.
2. Once access is granted, extract to
   `crates/dsfb-robotics/data/icub_pushrecovery/`.
3. Run `paper-lock icub_pushrecovery`.

## Residual construction — dual channel

Humanoid balancing exposes two complementary residual channels. DSFB
combines them via
[`balancing::combine_channels`](../src/balancing.rs) with
[`icub_pushrecovery::DEFAULT_COMBINE`](../src/datasets/icub_pushrecovery.rs)
(`SumOfSquares`):

- **Contact-wrench residual
  `r_W(k) = ‖W_contact,measured(k) − W_contact,planned(k)‖`.**
  The whole-body controller plans a per-contact 6-D wrench each
  cycle; the measured wrench (from ankle/wrist F/T sensors) differs
  from the plan. The controller discards this residual
  post-balance-recovery.

- **Centroidal-momentum tracking residual
  `r_ξ(k) = ‖ξ_measured(k) − ξ_planned(k)‖`.** The balance
  controller's centroidal-momentum reference vs. what the observer
  estimates. Discarded once the feedback loop stabilises.

## Why the humanoid exemplar matters

Paper §10.10 pairs iCub with Cheetah 3 (§10.9) to span morphologies:
humanoid (higher DoF, bipedal balance) and quadruped. Reviewers
specifically expect a humanoid balance example because "balancing" in
robotics commonly evokes bipedal / humanoid control; the pairing
prevents a reviewer from dismissing DSFB's balancing family as
"quadruped-only".

## DSFB bounded claim

Paper §10.10 bounded claim: *DSFB identifies structured residual
episodes during humanoid push-recovery distinguishable from nominal
balance-controller tracking error*. DSFB does **not** claim better
balance, earlier fall prediction, or any control-performance
improvement.

## Reproducibility

- Headline numbers require the iCub DUA corpus under
  `paper-lock icub_pushrecovery`.
- Smoke-test: `paper-lock icub_pushrecovery --fixture` uses the
  6-sample dual-channel fixture for deterministic CI verification.
