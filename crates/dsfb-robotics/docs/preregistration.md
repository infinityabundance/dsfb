# Stage III Pre-Registration — `dsfb-robotics`

This document records the methodological freeze of the DSFB Stage III
empirical protocol implemented in this crate. All numbers in §10 of
the companion paper (`paper/dsfb_robotics.tex`) are emitted under
this protocol; any future addition (new dataset, new fixture, etc.)
runs under the **same** parameters without retroactive tuning.

## Frozen protocol parameters

| Parameter | Value | Where defined |
|---|---|---|
| Drift window $W$ | **8** | [`src/paper_lock.rs`](../src/paper_lock.rs) `PAPER_LOCK_W` |
| Grazing window $K$ | **4** | [`src/paper_lock.rs`](../src/paper_lock.rs) `PAPER_LOCK_K` |
| Boundary fraction $\beta$ | **0.5** | [`src/envelope.rs`](../src/envelope.rs) default |
| Slew threshold $\delta_s$ | **0.05** | [`src/envelope.rs`](../src/envelope.rs) default |
| Calibration window | **first 20 % of finite samples** | [`src/paper_lock.rs::calibrated_envelope`](../src/paper_lock.rs) |
| Envelope $\rho$ | **mean + 3 × stddev** of calibration window | [`src/envelope.rs::calibrate_from_window`](../src/envelope.rs) |
| Hysteresis | **2 confirmations to commit a state change** | [`src/grammar.rs::evaluate`](../src/grammar.rs) |
| Bootstrap | 1000 replicates × stationary block, $L = W = 8$ | [`scripts/bootstrap_census.py`](../scripts/bootstrap_census.py) |

These are pinned by the const generics on `DsfbRoboticsEngine<W, K>`
and the constants in `paper_lock.rs`. Changing them requires
modifying source code, which is mechanically visible in `git log`.

## Frozen dataset slate

Twenty real-world physical-hardware datasets, listed in `paper/dsfb_robotics.tex`
§10.1–§10.20 and enumerated in [`src/main.rs::SUPPORTED_SLUGS`](../src/main.rs).

Three rows ship literal published-model torque-domain residuals
(`panda_gaz`, `dlr_justin`, `ur10_kufieta`); one row ships a
kinematic-domain residual matching what its open upstream bundle
distributes (`kuka_lwr`); the remaining sixteen rows ship
deviation-from-calibration-window-nominal residuals on the
recorded sensor channel canonical to each dataset.

## What this freeze means

- The Stage III parameter set $(W, K, \beta, \delta_s)$ is fixed for
  the slate. Any future dataset added to the slate runs under these
  same parameters.
- The bootstrap, sensitivity-grid, and ablation protocols defined in
  §Sensitivity / §Bootstrap CI / §Ablation are fixed for the slate;
  no per-dataset tuning is permitted.
- The 20-dataset slate is the slate. Adding a 21st dataset is
  permitted; removing a row from the existing 20 requires a tagged
  release with a documented justification (e.g., upstream takedown,
  licence change).

## What this freeze does not promise

- It does not promise the 20-dataset numbers will not change. Bug
  fixes to the FSM, residual computations, or dataset preprocessors
  produce different numbers. Each such change is a tagged release
  whose `CHANGELOG.md` records the delta.
- It does not promise the parameter set $(W, K, \beta, \delta_s)$ is
  optimal in any sense. Tier 6 sensitivity analysis quantifies
  elasticity; the chosen values are the canonical defaults the paper
  reports against.
- It does not constrain non-Stage-III work (dsfb-rf, dsfb-database,
  dsfb-gray, etc.) — each crate's pre-registration is its own
  document.

## Verification

To verify the protocol parameters at any commit:

```bash
grep -E 'PAPER_LOCK_W|PAPER_LOCK_K|boundary_frac|delta_s' \
    crates/dsfb-robotics/src/paper_lock.rs \
    crates/dsfb-robotics/src/envelope.rs
```

The matching tag is `paper-lock-protocol-frozen-v1` (created at
the commit landing this document). Subsequent freezes get
incrementing version suffixes.

## Why pre-registration matters here

The DSFB framework explicitly does not compete with incumbent
methods on accuracy / lead-time / detection metrics. It does
report a bounded structural surface (compression ratio, Boundary
count, Violation count) that downstream practitioners may consume.
For those reports to be trustworthy, the parameters that produce
them must be fixed before each new measurement; otherwise any
observed structural pattern could be a result of post-hoc tuning.
This pre-registration removes that degree of freedom from the
analysis.
