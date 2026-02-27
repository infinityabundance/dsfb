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


def ensure_chrome_for_kaleido(pio_module=None) -> str:
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
        pio = pio_module
        if pio is None:
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

def prepare_plot_packages(target_dir: Path) -> str:
    if target_dir.exists():
        shutil.rmtree(target_dir)
    target_dir.mkdir(parents=True, exist_ok=True)

    subprocess.run(
        [
            sys.executable,
            "-m",
            "pip",
            "install",
            "-q",
            "--upgrade",
            "--target",
            str(target_dir),
            "pandas",
            "plotly>=6.1.1,<7",
            "kaleido>=1.0.0,<2",
        ],
        check=True,
    )

    return str(target_dir)


def load_plotly_from_site_packages(site_packages: str):
    if site_packages in sys.path:
        sys.path.remove(site_packages)
    sys.path.insert(0, site_packages)

    for module_name in list(sys.modules):
        if module_name == "plotly" or module_name.startswith("plotly."):
            sys.modules.pop(module_name, None)
        if module_name == "_plotly_utils" or module_name.startswith("_plotly_utils."):
            sys.modules.pop(module_name, None)
        if module_name == "kaleido" or module_name.startswith("kaleido."):
            sys.modules.pop(module_name, None)

    import kaleido
    import plotly
    import plotly.graph_objects as go
    import plotly.io as pio

    return plotly, go, pio, kaleido


PLOT_PACKAGE_DIR = (
    Path("/content/.dsfb-ddmf-plot-packages")
    if "google.colab" in sys.modules
    else REPO_ROOT / ".dsfb-ddmf-plot-packages"
)
PLOT_PYTHON = sys.executable
PLOT_SITE_PACKAGES = prepare_plot_packages(PLOT_PACKAGE_DIR)
print(f"Using plotting Python: {PLOT_PYTHON}")
print(f"Using plotting package dir: {PLOT_SITE_PACKAGES}")

import importlib.metadata as importlib_metadata

plotly_module, _, pio_module, kaleido_module = load_plotly_from_site_packages(PLOT_SITE_PACKAGES)

print(
    f"Using Plotly version: {getattr(plotly_module, '__version__', importlib_metadata.version('plotly'))}"
)
print(
    f"Using Kaleido version: {getattr(kaleido_module, '__version__', importlib_metadata.version('kaleido'))}"
)

CHROME_BIN = ensure_chrome_for_kaleido(pio_module)
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
import tempfile

plotly_module, go, _, kaleido_module = load_plotly_from_site_packages(PLOT_SITE_PACKAGES)

print(
    f"Plotly image export ready with Plotly {getattr(plotly_module, '__version__', 'unknown')}"
    f" and Kaleido {getattr(kaleido_module, '__version__', 'unknown')}"
)

results = pd.read_csv(out_dir / "results.csv")
impulse = pd.read_csv(out_dir / "single_run_impulse.csv")
persistent = pd.read_csv(out_dir / "single_run_persistent.csv")

results["effective_amplitude"] = results["D"].where(results["D"].abs() > 0.0, results["B"].abs())


def save_plot(fig: go.Figure, stem: str) -> None:
    html_path = out_dir / f"{stem}.html"
    png_path = out_dir / f"{stem}.png"
    pdf_path = out_dir / f"{stem}.pdf"
    fig.write_html(html_path)

    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as fig_file:
        fig_file.write(fig.to_json())
        fig_json_path = Path(fig_file.name)

    export_script = """
import os
import sys
from pathlib import Path

site_packages = sys.argv[5]
if site_packages not in sys.path:
    sys.path.insert(0, site_packages)

import plotly.io as pio

fig_json_path = Path(sys.argv[1])
png_path = Path(sys.argv[2])
pdf_path = Path(sys.argv[3])
browser_path = sys.argv[4]

if browser_path:
    os.environ["BROWSER_PATH"] = browser_path

fig = pio.from_json(fig_json_path.read_text())
pio.write_image(fig, png_path, format="png", scale=2)
pio.write_image(fig, pdf_path, format="pdf")
"""

    try:
        export_env = os.environ.copy()
        existing_pythonpath = export_env.get("PYTHONPATH", "")
        export_env["PYTHONPATH"] = (
            f"{PLOT_SITE_PACKAGES}:{existing_pythonpath}"
            if existing_pythonpath
            else PLOT_SITE_PACKAGES
        )
        subprocess.run(
            [
                PLOT_PYTHON,
                "-c",
                export_script,
                str(fig_json_path),
                str(png_path),
                str(pdf_path),
                os.environ.get("BROWSER_PATH", ""),
                PLOT_SITE_PACKAGES,
            ],
            check=True,
            capture_output=True,
            text=True,
            env=export_env,
        )
        print(f"Saved {html_path.name}, {png_path.name}, and {pdf_path.name}")
    except subprocess.CalledProcessError as exc:
        print(f"Saved {html_path.name}")
        print(f"Static export failed for {stem}; continuing without PNG/PDF.")
        if exc.stderr:
            print(exc.stderr.strip())
    finally:
        fig_json_path.unlink(missing_ok=True)


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
