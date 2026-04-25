# Colab reproduction

This directory contains a one-click Google Colab notebook that reproduces
every figure in the companion paper's §10 using the in-crate micro-fixtures
and the same `paper-lock` binary that the host-side
[`scripts/figures_real.py`](../scripts/figures_real.py) uses.

## One-click link

After the repository is published, the notebook will be reachable at:

> `https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-robotics/colab/dsfb_robotics_reproduce.ipynb`

Locally you can open the notebook directly with Jupyter:

```bash
jupyter notebook colab/dsfb_robotics_reproduce.ipynb
```

## What the notebook does

1. **Build `paper-lock`** from source using the pinned Rust toolchain
   (`rust-toolchain.toml`) with `--features std,paper_lock`.
2. **Invoke `paper-lock --fixture --emit-episodes`** for each of the
   ten dataset slugs. The full smoke-test run takes ≈ 5 s of pure-CPU
   time on free-tier Colab.
3. **Parse the JSON output** and render the same figures as
   `scripts/figures_real.py` inline in the notebook.
4. **Display** the compression histogram and per-dataset grammar
   timelines for the reviewer.

Total end-to-end wall time on free-tier Colab: **under 5 minutes**,
dominated by the one-off Rust build. Subsequent cell re-runs in the
same session finish in seconds.

## Honesty disclosure

The notebook intentionally runs against the in-crate micro-fixtures
(smoke-test data). The paper's headline numbers in §10 come from
real-data runs of `paper-lock` against the full public datasets —
those require the corpora to be fetched separately under each
dataset's upstream licence / data-use agreement (see each
`docs/<slug>_oracle_protocol.md`). The notebook includes a visible
banner in its first markdown cell explaining this.

## Bit-exact reproduction

Three consecutive runs of the notebook produce **byte-identical** JSON
output from every `paper-lock --fixture` invocation — the same
deterministic tolerance gate enforced by
[`tests/paper_lock_binary.rs`](../tests/paper_lock_binary.rs)'s
`fixture_output_is_bit_exact_across_repeat_invocations` test.

## Dependencies

- A working Rust toolchain (Colab installs it automatically when the
  first cell runs).
- Python 3.11+ with `matplotlib` and `numpy` (pre-installed on Colab).

No other runtime dependencies are introduced by the notebook; it
re-uses `scripts/figures_real.py` verbatim.
