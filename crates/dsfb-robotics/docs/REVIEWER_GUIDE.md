# 30-Minute Reviewer Guide — `dsfb-robotics`

This guide walks a reviewer from `git clone` to deep understanding of
the empirical claims in approximately 30 minutes. It is the
companion teaching artefact to [`scripts/reproduce.sh`](../scripts/reproduce.sh)
and [`README_REVIEWER.md`](../README_REVIEWER.md), with concrete numbers
to expect and explicit reading rules at each step.

## What you will verify (in order)

1. The crate ships **20 real-world physical-hardware datasets** with
   zero simulation data.
2. Three of the four kinematic-arm rows ship **literal published-model
   torque-domain residuals** (`panda_gaz`, `dlr_justin`, `ur10_kufieta`);
   the fourth (`kuka_lwr`) ships a kinematic-domain residual matching
   what the open Simionato bundle distributes.
3. The DSFB grammar census on every dataset is **bit-exact across
   repeat invocations** of the production Rust binary (and across
   architectures, per the CI workflow).
4. Every headline census number in §10 is reported with a **95 %
   bootstrap confidence interval** computed by driving the production
   Rust binary 1000 times per dataset on stationary-block-bootstrap
   resamples.
5. The framework is **silent on datasets where it should be silent**
   (the silent-augment rows: `openx`, `unitree_g1`, `droid`, plus the
   ping-pong row), and surfaces structural episodes on rows where
   structure exists.
6. **No row ever claims to outperform the incumbent.** The
   per-dataset incumbent-relationship table makes this explicit.

## Recommended reading order

### Minute 0–5: Open the dashboard

Open [`paper/dashboard.html`](../paper/dashboard.html) in a browser.
The dashboard loads `panda_gaz` automatically. You should see:

- **Top bar**: `panda_gaz · residual = published-theta · total samples = 20,544 · envelope ρ = 27.652`. The `published-theta` tag means this row's residual is the literal Gaz 2019 dynamic-model residual — `r(k) = ‖τ_meas(k) − τ_predicted(q, q̇, q̈; θ̂_Gaz)‖` — computed by running the vendored cpp model on the recorded trajectory.
- **Bootstrap line**: `Boundary 1898 [1614,2180], Violation 162 [101,234], compression 0.100 [0.085,0.116]`. This is the headline census with its 95 % bootstrap CI.
- **Grammar timeline plot**: bar colours green/yellow/red = Admissible/Boundary/Violation. The trace should be predominantly green (90 %), with concentrated yellow/red bursts at the high-acceleration moments of the excitation trajectory.
- **Residual + envelope plot**: black line = ‖r(k)‖, red dash = ρ, yellow dot = βρ. The trace should cross βρ episodically and ρ rarely.

Click `cwru` in the sidebar. You should see a much shorter trace
(151 samples) where Violations dominate (118 of 151 timesteps). Note
the dashboard says `residual = early-window-nominal` for cwru — this is
correct: `cwru` is a PHM bearing dataset where the residual is the
deviation of the BPFI envelope-spectrum amplitude from the calibrated
healthy-window baseline.

Click `openx` to see the silent-augment posture in action: dominant
green Admissible regime, sparse yellow Boundary at the rotation
transitions only, no red Violations. **This is what augmentation looks
like when there is nothing to augment.**

### Minute 5–10: Run `paper-lock` once

Open a terminal in the crate directory:

```bash
cargo build --release --features std,paper_lock --bin paper-lock
target/release/paper-lock panda_gaz | tee /tmp/panda_gaz.json
```

Expected stderr line: `paper-lock: panda_gaz residual definition = published-theta (data/processed/panda_gaz_published.csv)`.

Expected stdout JSON aggregate (verbatim):

```json
{
  "total_samples": 20544,
  "admissible": 18489,
  "boundary": 1882,
  "violation": 173,
  "compression_ratio": 0.10002920560747663,
  "max_residual_norm_sq": 3510.4484291194285
}
```

Run it three times and SHA-256 the outputs; they should match.

```bash
sha256sum /tmp/panda_gaz.json
target/release/paper-lock panda_gaz > /tmp/panda_gaz_2.json
diff /tmp/panda_gaz.json /tmp/panda_gaz_2.json   # silent
```

### Minute 10–15: Inspect the published-θ̂ machinery

Look at how `panda_gaz`'s residual stream is computed:

```bash
ls data/panda_gaz/upstream_model/    # vendored Gaz 2019 cpp + LICENCE.upstream
ls data/panda_gaz/Exciting_Traj/Trajectory_1/rbt_log/   # raw recorded q/dq/τ
head -5 data/processed/panda_gaz_published.csv          # 20544 rows of ‖r‖
```

Re-run the cpp model on the recorded trajectory:

```bash
bash scripts/build_panda_gaz_model.sh
python3 scripts/compute_published_residuals.py panda_gaz
md5sum data/processed/panda_gaz_published.csv
# rerun and compare — should be bit-exact
python3 scripts/compute_published_residuals.py panda_gaz
md5sum data/processed/panda_gaz_published.csv
```

Same drill for UR10 (Pinocchio + URSim URDF):

```bash
python3 scripts/compute_published_residual_ur10.py
head -5 data/processed/ur10_kufieta_published.csv
```

### Minute 15–20: Inspect the bootstrap + sensitivity + ablation

```bash
ls audit/bootstrap/   # 20 *_ci.json files
cat audit/bootstrap/panda_gaz_ci.json | python3 -m json.tool | head -40
```

Each `_ci.json` records: residual_source (literal or proxy), 1000
replicates, the point estimate (run on the original stream), and the
mean and 95 % percentile-bootstrap CI for each census quantity. The
bootstrap engine = the production Rust binary driven via subprocess.

```bash
cat audit/sensitivity/panda_gaz_summary.json   # elasticity per parameter
cat audit/ablation/panda_gaz_ablation.json     # drift / slew / hysteresis ablation
```

Expect: `β` is the dominant sensitivity knob (compression spread
0.39); `W` and `δ_s` are essentially insensitive (spread ≤ 0.002);
hysteresis is the load-bearing FSM component (suppresses 966
single-sample state flips on `panda_gaz` when removed).

### Minute 20–25: Honest-negatives check

Open [`paper/dsfb_robotics.tex`](../paper/dsfb_robotics.tex) and grep for
the honest-negatives section:

```bash
grep -n 'Where DSFB Adds Nothing\|silent-augment' paper/dsfb_robotics.tex
```

Read the section §10.X "Where DSFB Adds Nothing Structurally". It
explicitly names four datasets (`openx`, `unitree_g1`, `droid`,
`aloha_static_pingpong_test`) where DSFB is correctly silent. Then
read §10.Y "Per-Dataset Incumbent-Relationship Table" — every row is
labelled "orthogonal", "literal-augmentation", or "silent-augment";
**no row ever says "outperforms"**.

Verify mechanically:

```bash
grep -i 'outperform\|beat\|earlier than\|faster than' paper/dsfb_robotics.tex
# expected: empty
```

### Minute 24–25: Operator-facing flags (`--explain`, `--emit-review-csv`)

Two operator-facing flags are first-class in the production binary:

```bash
target/release/paper-lock panda_gaz --explain | tail -20
```

`--explain` adds an `explain[]` array to the JSON output. Each entry is a
post-commit narrative for one episode: the structural condition that
fired the FSM transition (`sustained-outward-drift` /
`abrupt-slew` / `recurrent-grazing`), the sample index, and the band the
residual entered (`(βρ, ρ]` for Boundary, `> ρ` for Violation). Read it
as a triage prompt — "the FSM committed Boundary at sample $k$ under
this structural condition; an operator should now look at the raw
residual and the upstream controller log at this index" — not as a
fault explanation. (See [`docs/DEPLOYMENT_ANTIPATTERNS.md`](DEPLOYMENT_ANTIPATTERNS.md)
anti-pattern 4 for the full reading rule.)

```bash
target/release/paper-lock panda_gaz --emit-review-csv --review-csv-path /tmp/panda_gaz_review.csv
head /tmp/panda_gaz_review.csv
```

`--emit-review-csv` writes one row per Boundary / Violation episode in a
spreadsheet-friendly schema (`start, end, length, label, reason_code,
peak_norm`) suitable for piping into a triage queue or operator
dashboard. The CSV is independent of the JSON output and is deliberately
narrow — the JSON remains the canonical schema-validated artefact, the
CSV is the operator handoff.

### Minute 25–30: The non-claim discipline

Read §11 "Worked Example: Augmenting a Gaz 2019 Identification Report"
end-to-end. The chapter walks through what Gaz 2019 already ships, what
DSFB adds alongside, and explicitly enumerates what the augmentation
**does not** claim:

- Does not improve Gaz's parameter fit
- Does not reduce $\sigma_{\text{noise}}$
- Does not accelerate identification
- Does not find a fault Gaz's pipeline missed
- Does not substitute for any element of the Gaz method

Read §11.4 (the four-arm summary table). The augmentation posture
generalises across all four kinematic-arm rows with matching
non-claim discipline.

## What you should walk away with

- The crate ships 20 real-world public datasets.
- The DSFB framework consumes the residuals already produced by
  existing identification / monitoring / control pipelines.
- The framework emits a deterministic, bit-exact, reproducibility-
  audited grammar timeline alongside each pipeline's existing summary
  statistic, never replacing it.
- The framework is silent on datasets where it should be silent.
- The paper makes zero "outperforms" claims and the codebase has zero
  tests that would pass if such a claim were silently introduced (the
  orthogonality property test in [`tests/proptest_orthogonality.rs`](../tests/proptest_orthogonality.rs)).

## If something doesn't match

If your local SHA-256 of `paper-lock <slug>` output differs from
[`audit/checksums.txt`](../audit/checksums.txt), the cause is not the
algorithm. Check (in this order):

1. Your toolchain pin: `rust-toolchain.toml` requires Rust 1.85.1.
   Run `rustc --version`; if it differs, run `rustup toolchain install 1.85.1`.
2. Your raw data fetch: `python3 scripts/preprocess_datasets.py` should
   exit 0 on every dataset. A partial download will fail loudly.
3. Your floating-point model: the crate assumes IEEE 754 binary64.
   No other arithmetic semantic should appear on x86_64 / aarch64.

Open an issue with your `audit/checksums.fresh.txt` file and your
platform details.
