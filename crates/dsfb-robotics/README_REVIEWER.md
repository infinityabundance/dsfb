# Reviewer Reproduction Guide — `dsfb-robotics`

This is a single-page guide for reviewers to reproduce every empirical
number in the companion paper end-to-end on a clean machine. Each step
is idempotent.

## One-shot reproduction

```bash
nix develop          # pinned Rust 1.85.1 + Python 3.12 + LaTeX + Eigen
bash scripts/reproduce.sh
```

The script runs nine steps:

1. **Build the vendored Gaz 2019 cpp dynamic model**
   (`data/panda_gaz/upstream_model/`, the literal C++ from
   `marcocognetti/FrankaEmikaPandaDynModel`) so the published-$\hat\theta$
   residual stream for `panda_gaz` can be computed.

2. **Preprocess all 20 datasets** —
   `scripts/preprocess_datasets.py` reads the raw vendored data under
   `data/<slug>/` and writes per-slug `data/processed/<slug>.csv`.
   Existing CSVs are skipped.

3. **Compute the published-$\hat\theta$ residual stream** for
   `panda_gaz` by running the cpp model on the recorded
   $(q, \dot q)$ trajectory — emits
   `data/processed/panda_gaz_published.csv`.

4. **Build the `paper-lock` release binary** under
   `target/release/paper-lock`.

5. **Run `paper-lock` on every dataset** and SHA-256-checksum each JSON
   output. If `audit/checksums.txt` exists, the script diffs against it
   and reports any deviation. On first run it emits
   `audit/checksums.txt` for future comparison.

6. **Bootstrap CI sweep** — 1000-replicate stationary block bootstrap
   on every dataset, driving the production Rust binary via
   `--csv-path`. Output: `audit/bootstrap/<slug>_ci.json`.

7. **Sensitivity grid** — 300-cell parameter sweep on `panda_gaz`.
   Output: `audit/sensitivity/panda_gaz_grid.csv` and `_summary.json`.

8. **Ablation study** — three component-disable runs on `panda_gaz`,
   `cwru`, and `icub_pushrecovery`. Output:
   `audit/ablation/<slug>_ablation.json`.

9. **Compile the paper PDF** at `paper/dsfb_robotics.pdf`.

## Step-by-step reproduction

If you'd rather do each step manually:

```bash
bash scripts/build_panda_gaz_model.sh
python3 scripts/preprocess_datasets.py
python3 scripts/compute_published_residuals.py
cargo build --release --features std,paper_lock --bin paper-lock
python3 scripts/bootstrap_census.py
python3 scripts/sensitivity_grid.py panda_gaz
python3 scripts/ablation.py
(cd paper && latexmk -pdf dsfb_robotics.tex)
```

## Without Nix

If you don't have Nix, the pre-requisites are:

- Rust 1.85.1 (`rust-toolchain.toml` pins this)
- Python 3.12 with `numpy h5py pyarrow matplotlib plotly scipy`
- A C++11 compiler, CMake ≥ 3.5, Eigen 3
- A LaTeX distribution with `latexmk` and the standard scientific
  packages (TeXLive `scheme-medium` is sufficient)
- Optional: Valgrind for the allocation budget audit

## What gets produced

After a successful run:

| Path | What |
|---|---|
| `paper/dsfb_robotics.pdf` | The full 56-page paper |
| `target/release/paper-lock` | The production CLI |
| `data/processed/<slug>.csv` × 20 | Preprocessed residual streams |
| `data/processed/panda_gaz_published.csv` | Literal Gaz-$\hat\theta$ residual stream |
| `audit/bootstrap/<slug>_ci.json` × 20 | 95 % bootstrap CIs |
| `audit/sensitivity/panda_gaz_grid.csv` | 300-cell sensitivity grid |
| `audit/ablation/<slug>_ablation.json` × 3 | Component-ablation deltas |
| `audit/uncertainty/<slug>_budget.json` × 20 | GUM uncertainty budgets |
| `audit/checksums.txt` | SHA-256 checksums for every JSON output |

## Determinism guarantee

The paper-lock binary is bit-exact across repeat invocations and across
architectures (verified by the
[`.github/workflows/determinism.yml`](.github/workflows/determinism.yml)
matrix on Linux x86_64 / Linux aarch64 / macOS aarch64). If your local
output diffs from `audit/checksums.txt`, the cause is *not* the
algorithm — investigate your toolchain pin (`rust-toolchain.toml`),
your data fetch (a corrupted or partial download will fail loudly in
step 2), or your floating-point model (we assume IEEE 754 binary64).

## Reproducibility receipts

Two attestation artefacts cover this revision:

1. **Per-dataset paper-lock JSON checksums** at
   [`audit/checksums.txt`](audit/checksums.txt) (20 entries, one per
   dataset). The `reproduce.sh` script re-emits these on every run and
   diffs against the committed file.
2. **dsfb-gray DSSE attestation** — running
   `cargo run -p dsfb-gray --release --bin dsfb-scan-crate -- crates/dsfb-robotics`
   produces `dsfb_robotics_scan.dsse.json` (unsigned DSSE-format
   attestation of the source-visible audit) under `audit/dsfb-gray-*/`.
   The SHA-256 of this artefact for the canonical revision is committed
   at the bottom of `audit/checksums.txt`. Reviewers can verify the
   audit was run on the canonical commit by matching their freshly-
   generated DSSE to that hash.

For an additional cross-architecture sanity check before the CI run,
follow the local QEMU protocol in
[`docs/cross_arch_qemu.md`](docs/cross_arch_qemu.md).

## Pre-registered protocol

The Stage III parameter set $(W=8, K=4, \beta=0.5, \delta_s=0.05)$,
the bootstrap protocol, the sensitivity grid, and the 20-dataset
slate are all frozen at this commit. See
[`docs/preregistration.md`](docs/preregistration.md). Future
additions to the slate run under the same parameters without
retroactive tuning. The freeze tag is `paper-lock-protocol-frozen-v1`.

## External replication invitation

If you have run `bash scripts/reproduce.sh` end-to-end on hardware
not in the CI matrix, the project welcomes your replication record.
Open a PR adding an entry under `audit/replication/` of the form
`audit/replication/<institution>_<arch>.md` containing:

- Your platform (OS / kernel / CPU / arch)
- Your Rust toolchain version (`rustc --version`) and Python version
- The SHA-256 of every `audit/json_outputs/*.json` you produced
- A diff of your `audit/checksums.fresh.txt` against the committed
  `audit/checksums.txt`
- Any environmental detail that may explain residual divergence
  (e.g., Linux distribution, glibc version, FP rounding-mode tweaks)

Replication records are first-class artefacts of this work; they
are committed to the repository, attributed in the paper's
acknowledgements section on the next tagged release, and referenced
by the cross-architecture determinism claim. The intent is to make
external replication a recurring contribution path rather than a
one-time exercise.
