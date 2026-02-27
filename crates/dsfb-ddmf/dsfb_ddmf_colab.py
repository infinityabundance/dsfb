# %% [markdown]
# # DDMF Monte Carlo Notebook
#
# This notebook demonstrates the `dsfb-ddmf` crate, a deterministic disturbance-side
# extension of the `dsfb` workspace for residual-envelope fusion systems.
#
# What it does:
# - builds and runs the Rust `dsfb-ddmf` crate
# - executes the default x360 Monte Carlo disturbance sweep
# - loads the generated CSV outputs into pandas
# - plots envelope and trust behavior for impulse and persistent-elevated examples
# - plots Monte Carlo summary behavior across disturbance regimes
# - saves Plotly figures back into the active output directory as PNG and PDF
#
# Why it exists:
# - to show how deterministic disturbance classes affect the residual envelope `s[n]`
# - to show how trust weights `w[n] = 1 / (1 + beta * s[n])` suppress or recover
# - to make the DDMF paper behavior easy to inspect in Colab without re-implementing
#   the Rust simulation logic in Python
#
# How it works:
# - the Rust binary samples seeded disturbance cases across bounded, drift, slew-only,
#   impulsive, and persistent-elevated regimes
# - each run evaluates residuals `r[n] = epsilon[n] + d[n]`
# - the envelope recursion updates as `s[n+1] = rho * s[n] + (1-rho) * abs(r[n])`
# - trust is computed from the current envelope and exported for plotting
#
# Where outputs go:
# - the CLI writes to `output-dsfb-ddmf/YYYYMMDD_HHMMSS/` under the repo root
# - this notebook auto-detects the latest such directory and uses it as `out_dir`
#
# Main Monte Carlo / model parameters:
# - `runs` / `n_runs`: number of Monte Carlo cases; default here is x360
# - `n_steps`: number of time steps per simulation run
# - `seed`: RNG seed for reproducible disturbance parameter sampling
# - `rho`: envelope forgetting factor in `(0, 1)`; larger means slower decay / longer memory
# - `beta`: trust sensitivity; larger means trust falls faster as envelope grows
# - `epsilon_bound`: deterministic bound on the residual contribution `epsilon[n]`
# - `recovery_delta`: tolerance used when deciding whether an impulsive case recovered
#
# Disturbance-specific parameters visible in `results.csv`:
# - `D`: disturbance magnitude column for bounded / impulsive / persistent-elevated cases
# - `B`: drift-rate-like column used for drift cases and the nominal level for persistent-elevated cases
# - `S`: slew or rate-bound parameter
# - `impulse_start`: first index of an impulse window
# - `impulse_len`: impulse duration in samples
# - `s0`: initial envelope state for the run
#
# Output summary columns:
# - `max_envelope`: peak `s[n]` observed in the run
# - `min_trust`: minimum trust reached in the run
# - `time_to_recover`: first recovery index when applicable; `-1` means not recoverable / not observed
# - `regime_label`: qualitative regime such as `bounded_nominal`, `persistent_elevated`,
#   `impulsive`, or `unbounded`

# %%
from pathlib import Path
import subprocess
import sys


def detect_repo_root() -> Path:
    candidates = [Path("/content/dsfb-ddmf"), Path("/content/dsfb"), Path.cwd()]
    for candidate in candidates:
        if (candidate / "Cargo.toml").exists():
            return candidate
    raise FileNotFoundError("Could not locate the Rust repo root.")


REPO_ROOT = detect_repo_root()
CRATE_DIR = REPO_ROOT / "crates" / "dsfb-ddmf"
if not CRATE_DIR.exists():
    CRATE_DIR = REPO_ROOT

subprocess.run(
    [sys.executable, "-m", "pip", "install", "-q", "pandas", "plotly", "kaleido"],
    check=True,
)

# %%
subprocess.run(["cargo", "build", "--release"], cwd=CRATE_DIR, check=True)
subprocess.run(
    ["cargo", "run", "--release", "--bin", "monte_carlo", "--", "--runs", "360"],
    cwd=CRATE_DIR,
    check=True,
)

# %% [markdown]
# The CLI writes all outputs under the repo-root `output-dsfb-ddmf/` directory.
# We always pick the lexicographically latest timestamped subdirectory.

# %%
OUTPUT_ROOT = REPO_ROOT / "output-dsfb-ddmf"
RUN_DIRS = sorted(path for path in OUTPUT_ROOT.iterdir() if path.is_dir())
if not RUN_DIRS:
    raise FileNotFoundError("No output directories found under output-dsfb-ddmf/.")

out_dir = RUN_DIRS[-1]
print(f"Using output directory: {out_dir}")

# %%
import pandas as pd

results = pd.read_csv(out_dir / "results.csv")
impulse = pd.read_csv(out_dir / "single_run_impulse.csv")
persistent = pd.read_csv(out_dir / "single_run_persistent.csv")

results["effective_amplitude"] = results["D"].where(results["D"].abs() > 0.0, results["B"].abs())

# %%
import plotly.graph_objects as go
import plotly.io as pio

fig1 = go.Figure()
fig1.add_trace(
    go.Scatter(
        x=impulse["n"],
        y=impulse["s"],
        mode="lines",
        name="Impulse",
        line=dict(width=3, color="#0f4c5c"),
    )
)
fig1.add_trace(
    go.Scatter(
        x=persistent["n"],
        y=persistent["s"],
        mode="lines",
        name="Persistent Elevated",
        line=dict(width=3, dash="dash", color="#c8553d"),
    )
)
fig1.update_layout(
    title="Envelope Evolution: Impulse vs Persistent Elevated",
    xaxis_title="n",
    yaxis_title="s[n]",
    template="plotly_white",
)
fig1.show()

pio.write_image(
    fig1,
    out_dir / "envelope_impulse_vs_persistent.png",
    format="png",
    scale=2,
)
pio.write_image(
    fig1,
    out_dir / "envelope_impulse_vs_persistent.pdf",
    format="pdf",
)

# %%
fig2 = go.Figure()
fig2.add_trace(
    go.Scatter(
        x=impulse["n"],
        y=impulse["w"],
        mode="lines",
        name="Impulse",
        line=dict(width=3, color="#1982c4"),
    )
)
fig2.add_trace(
    go.Scatter(
        x=persistent["n"],
        y=persistent["w"],
        mode="lines",
        name="Persistent Elevated",
        line=dict(width=3, dash="dash", color="#ff595e"),
    )
)
fig2.update_layout(
    title="Trust Evolution: Impulse vs Persistent Elevated",
    xaxis_title="n",
    yaxis_title="w[n]",
    template="plotly_white",
)
fig2.show()

pio.write_image(
    fig2,
    out_dir / "trust_impulse_vs_persistent.png",
    format="png",
    scale=2,
)
pio.write_image(
    fig2,
    out_dir / "trust_impulse_vs_persistent.pdf",
    format="pdf",
)

# %%
fig3 = go.Figure()
fig3.add_trace(
    go.Scatter(
        x=results["effective_amplitude"],
        y=results["max_envelope"],
        mode="markers",
        marker=dict(size=8, color=results["min_trust"], colorscale="Viridis", showscale=True),
        text=results["regime_label"],
        name="Monte Carlo runs",
    )
)
fig3.update_layout(
    title="Max Envelope vs Disturbance Amplitude",
    xaxis_title="disturbance amplitude",
    yaxis_title="max_envelope",
    template="plotly_white",
)
fig3.show()

pio.write_image(
    fig3,
    out_dir / "max_envelope_vs_amplitude.png",
    format="png",
    scale=2,
)
pio.write_image(
    fig3,
    out_dir / "max_envelope_vs_amplitude.pdf",
    format="pdf",
)

# %%
fig4 = go.Figure()
for regime_label in sorted(results["regime_label"].unique()):
    subset = results[results["regime_label"] == regime_label]
    fig4.add_trace(
        go.Histogram(
            x=subset["min_trust"],
            name=regime_label,
            opacity=0.6,
            nbinsx=24,
        )
    )

fig4.update_layout(
    title="Minimum Trust Distribution by Regime",
    xaxis_title="min_trust",
    yaxis_title="count",
    barmode="overlay",
    template="plotly_white",
)
fig4.show()

pio.write_image(
    fig4,
    out_dir / "min_trust_histogram.png",
    format="png",
    scale=2,
)
pio.write_image(
    fig4,
    out_dir / "min_trust_histogram.pdf",
    format="pdf",
)
