# %% [markdown]
# # DDMF Monte Carlo Notebook
#
# - This notebook visualizes deterministic envelope behavior under DDMF.
# - It compares impulse vs persistent elevated disturbances.
# - It summarizes Monte Carlo 360-degree disturbance sweeps.

# %% Cell 1: Install dependencies and prepare the repo/toolchain
from pathlib import Path
import os
import shutil
import subprocess
import sys


def detect_repo_root() -> Path:
    def is_workspace_root(path: Path) -> bool:
        return (path / "Cargo.toml").exists() and (path / "crates" / "dsfb-ddmf").exists()

    def is_crate_root(path: Path) -> bool:
        return (path / "Cargo.toml").exists() and (path / "src").exists() and path.name == "dsfb-ddmf"

    cwd = Path.cwd().resolve()

    for candidate in [cwd, *cwd.parents]:
        if is_workspace_root(candidate):
            return candidate
        if is_crate_root(candidate):
            return candidate.parent.parent

    content_root = Path("/content")
    if content_root.exists():
        for cargo_toml in content_root.glob("*/Cargo.toml"):
            candidate = cargo_toml.parent
            if is_workspace_root(candidate):
                return candidate
            if is_crate_root(candidate):
                return candidate.parent.parent

        for cargo_toml in content_root.glob("*/*/Cargo.toml"):
            candidate = cargo_toml.parent
            if is_workspace_root(candidate):
                return candidate
            if is_crate_root(candidate):
                return candidate.parent.parent

    raise FileNotFoundError(
        "Could not locate the Rust repo root. Clone the repository under /content and rerun the notebook."
    )


def prepare_repo_root() -> Path:
    if "google.colab" in sys.modules:
        repo_root = Path("/content/dsfb")
        if repo_root.exists():
            shutil.rmtree(repo_root)
        subprocess.run(
            [
                "git",
                "clone",
                "--depth",
                "1",
                "--branch",
                "main",
                "https://github.com/infinityabundance/dsfb.git",
                str(repo_root),
            ],
            check=True,
        )
        return repo_root

    return detect_repo_root()


def ensure_cargo() -> str:
    cargo = shutil.which("cargo")
    if cargo:
        return cargo

    cargo_home = Path.home() / ".cargo" / "bin" / "cargo"
    if cargo_home.exists():
        return str(cargo_home)

    subprocess.run(
        [
            "bash",
            "-lc",
            "curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal",
        ],
        check=True,
    )

    if cargo_home.exists():
        return str(cargo_home)

    raise FileNotFoundError("cargo is not available even after rustup installation.")


def ensure_chrome_for_kaleido() -> str:
    browser_candidates = [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
    ]

    for candidate in browser_candidates:
        browser_path = shutil.which(candidate)
        if browser_path:
            os.environ["BROWSER_PATH"] = browser_path
            return browser_path

    if "google.colab" in sys.modules:
        subprocess.run(
            [
                "bash",
                "-lc",
                "apt-get -qq update && (apt-get -qq install -y chromium-browser || apt-get -qq install -y chromium)",
            ],
            check=True,
        )

        for candidate in browser_candidates:
            browser_path = shutil.which(candidate)
            if browser_path:
                os.environ["BROWSER_PATH"] = browser_path
                return browser_path

    try:
        import plotly.io as pio

        browser_path = pio.get_chrome()
        if browser_path:
            os.environ["BROWSER_PATH"] = browser_path
            return browser_path
    except Exception:
        pass

    raise FileNotFoundError(
        "Could not find or install Chrome/Chromium for Kaleido static image export."
    )


REPO_ROOT = prepare_repo_root()
CRATE_DIR = REPO_ROOT / "crates" / "dsfb-ddmf"
if not CRATE_DIR.exists():
    CRATE_DIR = REPO_ROOT

commit_hash = subprocess.check_output(
    ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
    text=True,
).strip()
print(f"Using repository commit: {commit_hash}")

CARGO_BIN = ensure_cargo()
print(f"Using cargo binary: {CARGO_BIN}")

subprocess.run(
    [sys.executable, "-m", "pip", "install", "-q", "pandas", "plotly", "kaleido"],
    check=True,
)

CHROME_BIN = ensure_chrome_for_kaleido()
print(f"Using Chrome/Chromium binary: {CHROME_BIN}")

# %% Cell 2: Build and run the dsfb-ddmf CLI
subprocess.run([CARGO_BIN, "build", "--release"], cwd=CRATE_DIR, check=True)
subprocess.run(
    [CARGO_BIN, "run", "--release", "--bin", "monte_carlo", "--", "--runs", "360"],
    cwd=CRATE_DIR,
    check=True,
)

# %% Cell 3: Locate the latest output directory
OUTPUT_ROOT = REPO_ROOT / "output-dsfb-ddmf"
RUN_DIRS = sorted(path for path in OUTPUT_ROOT.iterdir() if path.is_dir())
if not RUN_DIRS:
    raise FileNotFoundError("No output directories found under output-dsfb-ddmf/.")

out_dir = RUN_DIRS[-1]
print(f"Using output directory: {out_dir}")

# %% Cell 4: Load CSVs and define save helper
import pandas as pd
import plotly.graph_objects as go
import plotly.io as pio

results = pd.read_csv(out_dir / "results.csv")
impulse = pd.read_csv(out_dir / "single_run_impulse.csv")
persistent = pd.read_csv(out_dir / "single_run_persistent.csv")

results["effective_amplitude"] = results["D"].where(results["D"].abs() > 0.0, results["B"].abs())


def save_plot(fig: go.Figure, stem: str) -> None:
    pio.write_image(fig, out_dir / f"{stem}.png", format="png", scale=2)
    pio.write_image(fig, out_dir / f"{stem}.pdf", format="pdf")


results.head()

# %% Cell 5: Envelope figure
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
save_plot(fig1, "envelope_impulse_vs_persistent")

# %% Cell 6: Trust figure
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
save_plot(fig2, "trust_impulse_vs_persistent")

# %% Cell 7: Monte Carlo summary figures
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
save_plot(fig3, "max_envelope_vs_amplitude")

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
save_plot(fig4, "min_trust_histogram")
