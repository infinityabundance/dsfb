# Bootstrap-coverage Monte Carlo (Pass-2 N4)

Quantifies the under-coverage of the 95% percentile-bootstrap
confidence interval that the §Live-Eval table reports at `n = 10`.

## Why

The §Live-Eval setup paragraph carries a literature caveat:

> With `n = 10`, the percentile-bootstrap 95% CI under-covers its
> nominal level on long-tailed metrics (FAR/hr in particular); CI
> widths in Table 2 should be read as lower bounds on true uncertainty.

The Pass-2 statistics reviewer (R4) asked us to **measure** the
under-coverage rather than just cite it. This experiment is the
measurement.

## What

For each of three source distributions chosen to mirror the metric
families in the live-eval table:

| Distribution         | Surrogate for | Tail behaviour      |
|----------------------|---------------|---------------------|
| Beta(8, 2)           | F1            | Bounded, mild skew  |
| Gamma(k=2, θ=0.3)    | TTD           | Right-skew          |
| log-Normal(μ=2, σ=1) | FAR/hr        | Heavy right tail    |

we draw `--n-mc` (default 2000) Monte-Carlo iterations per sample size
`n ∈ {5, 10, 20, 50}`. Per iteration, we compute the percentile-
bootstrap 95% CI on the sample mean using `B = 1000` resamples — the
same algorithm as `experiments/real_pg_eval/aggregate.py::bootstrap_ci`
that produces the table's CIs. Empirical coverage is the fraction of
iterations whose CI contains the known true mean.

## How

```sh
bash run.sh
```

≈ 2 seconds wall-clock. Pure synthetic; no engine, no podman, no fixtures.

Override knobs:

```sh
N_MC=5000 N_LIST=5,10,20,30,50,100 SEED=42 bash run.sh
```

## Output

* `out/coverage.csv` — one row per `(distribution, n)` pair with
  empirical coverage and mean CI width.
* `paper/figs/bootstrap_coverage.png` — coverage-vs-sample-size figure
  cited by paper §39.

## Indicative result (seed 42)

At `n = 10`:

* Beta(8, 2) (F1-like) ........... empirical coverage ≈ 0.89
* Gamma(2, 0.3) (TTD-like) ....... empirical coverage ≈ 0.88
* log-Normal(2, 1) (FAR/hr-like) . empirical coverage ≈ 0.80

The heavy-tailed case under-covers by ~15 percentage points, confirming
the qualitative literature claim and giving the §39 paragraph a
quantitative anchor.

## Determinism

Pure function of the CLI flags. The PRNG is documented (LCG identical
to `src/bin/baseline_tune.rs::Lcg`) and seeded from `--seed`. Two
independent toolchain builds produce byte-equal `coverage.csv`.
