"""Shared style, determinism, I/O, and figure-provenance plumbing for the DSFB figure gallery.

This module is the single foundation every figure-group module imports. Its jobs:

  * **Determinism.** Figures must re-render byte-for-byte identically (the campaign's figure-determinism
    gate). matplotlib otherwise stamps a creation date into PDFs and a Software tag into PNGs. We pin
    `SOURCE_DATE_EPOCH=0` (matplotlib reads it for the PDF CreationDate) *before* importing matplotlib,
    select the headless Agg backend, and write fixed `metadata=` on every `savefig`. No randomness is
    used anywhere; any layout that could be random (e.g. networkx spring layout) must pass a fixed seed.

  * **House style.** Serif 10 pt to match the paper body text, light grid, no top/right spines, DPI 140 —
    identical to the original `gen_figures.py` so new figures sit beside the existing ones seamlessly.

  * **Colour.** A colourblind-safe palette (Wong 2011) plus the DSFB grammar-token colours. Figures must be
    legible in greyscale and to colour-vision-deficient readers (part of the legendary rubric).

  * **Figure provenance.** Every figure saved via `save()` is recorded in a manifest (id -> source data ->
    caption -> sha256 of the PNG), written out as `figure_manifest.json`. This makes each figure an
    auditable artifact, mirroring the project's forensic-court ethos.

  * **Verbose logging.** `log()` writes to stdout *and* appends to the captured build-log file, so the
    crate command and the Colab notebook both produce the same human-readable record of what ran.
"""
import os

# Pin the PDF creation date to the epoch for reproducible PDF bytes. MUST be set before matplotlib import.
os.environ.setdefault("SOURCE_DATE_EPOCH", "0")

import csv
import glob
import hashlib
import json
import sys

import matplotlib

matplotlib.use("Agg")  # headless raster backend; no display server needed (CI / Colab / sandbox)
import matplotlib.pyplot as plt  # noqa: E402

# ── Paths ────────────────────────────────────────────────────────────────────────────────────────────
# CRATE = dsfb-chemical-engineering-edge ; WORKSPACE = repo root ; FIGDIR = paper/figures (git-tracked).
HERE = os.path.dirname(os.path.abspath(__file__))                       # .../scripts/figures
CRATE = os.path.dirname(os.path.dirname(HERE))                          # .../dsfb-chemical-engineering-edge
WORKSPACE = os.path.dirname(os.path.dirname(CRATE))                     # repo root
FIGDIR = os.path.join(WORKSPACE, "paper", "figures")
ATLAS_TOML = os.path.join(CRATE, "corpus", "chemometric_atlas.toml")
CUDA_REPORTS = os.path.join(WORKSPACE, "crates", "dsfb-chemical-engineering-cuda", "reports")
os.makedirs(FIGDIR, exist_ok=True)

# ── House style (matches the paper + the original gen_figures.py) ──────────────────────────────────────
plt.rcParams.update({
    "font.family": "serif",
    "font.size": 10,
    "axes.grid": True,
    "grid.alpha": 0.25,
    "axes.spines.top": False,
    "axes.spines.right": False,
    "figure.dpi": 140,
})

# ── Colour ────────────────────────────────────────────────────────────────────────────────────────────
# Wong (2011) colourblind-safe palette — used for all categorical encodings so figures read in greyscale
# and for colour-vision-deficient readers (legendary-rubric requirement).
CB = {
    "black": "#000000", "orange": "#E69F00", "skyblue": "#56B4E9", "green": "#009E73",
    "yellow": "#F0E442", "blue": "#0072B2", "vermillion": "#D55E00", "purple": "#CC79A7",
    "grey": "#999999",
}
# Semantic shortcuts used across figures.
INK = "#1d3557"       # primary line/ink
ACCENT = "#0072B2"    # primary categorical
WARN = "#D55E00"      # breach / fault / out-of-spec
OK = "#009E73"        # nominal / good
MUTE = "#9aa7b1"      # catalogued / silent / background

# DSFB grammar-token colours (kept consistent with gen_figures.py GRAMMAR_COLORS).
GRAMMAR_COLORS = {
    "NOM": "#dfe7ef", "DA": "#E69F00", "SS": "#D55E00", "EV": "#9b2226",
    "BG": "#a8c686", "RC": "#56B4E9", "CP": "#CC79A7", "SF": "#000000",
}

# Standing on-figure disclaimer fragments (academic honesty — every bounded claim shows its boundary).
DISCLAIMER = {
    "candidate": "Structural candidate only — not a root-cause or causal claim.",
    "advisory": "Advisory; read-only. No control or safety-instrumented-function authority.",
    "noncausal": "Temporal precedence + topology only — NOT a causal proof.",
    "similarity": "Structural similarity / retrieval hint — not identity, not causation.",
    "measured": "Measured on an NVIDIA RTX 4080 SUPER (CUDA 13.2). Determinism is exact; timing is hardware-specific.",
}

# ── Verbose build log + figure manifest ────────────────────────────────────────────────────────────────
_LOG_FH = None          # optional file handle for the captured build log
MANIFEST = []           # accumulates {id, group, source, caption, png_sha256}


def set_log(path):
    """Open `path` for appending the verbose build log (in addition to stdout)."""
    global _LOG_FH
    if path:
        _LOG_FH = open(path, "a")


def log(msg):
    """Print `msg` to stdout and, if a log file is open, append it there too (verbose build record)."""
    print(msg, flush=True)
    if _LOG_FH:
        _LOG_FH.write(msg + "\n")
        _LOG_FH.flush()


def _sha256_file(path):
    """Return the hex SHA-256 of a file's bytes (used to seal each rendered figure in the manifest)."""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def save(fig, fid, group, source, caption):
    """Save `fig` to FIGDIR as both PNG (raster) and PDF (vector), deterministically, and record provenance.

    `fid` is the figure id (also the file stem); `group` is its catalogue group letter; `source` is a short
    human description of the data it was rendered from; `caption` is the figure caption. The PNG's SHA-256
    is recorded in MANIFEST so the figure is an auditable artifact. Fixed `metadata=` + SOURCE_DATE_EPOCH
    make the bytes reproducible across runs.
    """
    png = os.path.join(FIGDIR, f"{fid}.png")
    pdf = os.path.join(FIGDIR, f"{fid}.pdf")
    fig.savefig(png, bbox_inches="tight", metadata={"Software": "dsfb-figures"})
    fig.savefig(pdf, bbox_inches="tight", metadata={"Creator": "dsfb-figures", "Producer": "dsfb-figures"})
    plt.close(fig)
    sha = _sha256_file(png)
    MANIFEST.append({"id": fid, "group": group, "source": source, "caption": caption, "png_sha256": sha})
    log(f"  [{group}] wrote {fid}.png/.pdf  sha256={sha[:12]}…  ({source})")


def have_graphviz():
    """True iff the graphviz `dot` binary is on PATH (graphs render via dot when present)."""
    import shutil
    return shutil.which("dot") is not None


def render_dot(dot_source, fid, group, source, caption, engine="dot"):
    """Render a graphviz DOT string to FIGDIR/<fid>.png + .pdf via the `dot` binary; record provenance.

    Graphviz produces far cleaner graph layouts than a hand-rolled matplotlib placement, and is the natural
    choice for the genuinely graph-structured figures (grammar FSM, topology, propagation, provenance). If
    `dot` is unavailable the caller is expected to provide a networkx fallback; here we log and skip so the
    rest of the gallery still renders. Output is deterministic for a fixed graphviz version + input.
    """
    import subprocess
    if not have_graphviz():
        log(f"  [{group}] SKIP {fid}: graphviz `dot` not on PATH (caller should supply a networkx fallback)")
        return False
    png = os.path.join(FIGDIR, f"{fid}.png")
    pdf = os.path.join(FIGDIR, f"{fid}.pdf")
    for fmt, out in (("png", png), ("pdf", pdf)):
        # -Gdpi pins raster density; feeding source on stdin keeps it deterministic (no temp-name leakage).
        cmd = [engine, f"-T{fmt}", "-Gdpi=140", "-o", out]
        subprocess.run(cmd, input=dot_source.encode(), check=True)
    sha = _sha256_file(png)
    MANIFEST.append({"id": fid, "group": group, "source": source, "caption": caption, "png_sha256": sha})
    log(f"  [{group}] wrote {fid}.png/.pdf (graphviz)  sha256={sha[:12]}…  ({source})")
    return True


def write_manifest(path, source_run=None):
    """Write the figure-provenance manifest (sorted by id) as pretty JSON.

    Deterministic by design: no wall-clock field (which would break the byte-identical re-render gate).
    Each entry already carries the PNG's SHA-256, so an identical render yields an identical manifest.
    """
    data = {
        "tool": "dsfb-chemical-engineering figure gallery",
        "source_run": os.path.basename(source_run.rstrip("/")) if source_run else None,
        "n_figures": len(MANIFEST),
        "figures": sorted(MANIFEST, key=lambda m: m["id"]),
    }
    with open(path, "w") as f:
        json.dump(data, f, indent=2)
    log(f"figure_manifest.json: {len(MANIFEST)} figures -> {path}")


# ── Data access helpers ─────────────────────────────────────────────────────────────────────────────────
def latest_run():
    """Return the most-recent edge demo output dir (lexicographic == chronological for the timestamps)."""
    runs = sorted(glob.glob(os.path.join(WORKSPACE, "output-dsfb-chemical-engineering", "*")))
    if not runs:
        sys.exit("no output run found; run the edge demo (or `dsfb-chem-edge figures`) first")
    return runs[-1]


def read_csv_rows(path):
    """Read a CSV into a list of dicts keyed by header; return [] if the file is absent (partial runs)."""
    if not os.path.exists(path):
        return []
    with open(path) as f:
        return list(csv.DictReader(f))


def ds_csv(run, dataset, name):
    """Path to a per-dataset CSV inside a run (e.g. residual_streams.csv)."""
    return os.path.join(run, "datasets", dataset, name)


def fnum(s, default=float("nan")):
    """Parse a float from a CSV cell, tolerating empty strings / 'na'."""
    if s is None or s == "" or s == "na":
        return default
    try:
        return float(s)
    except ValueError:
        return default


def figure_caption(ax, text):
    """Place a small italic on-figure disclaimer/caption below an axis (academic-honesty annotation).

    Uses an offset-points annotation anchored to the axis origin so it sits *below* the x-label and is
    captured by `bbox_inches="tight"` without colliding with the axis (a plain transAxes text overlaps the
    x-label on multi-panel figures).
    """
    ax.annotate(text, xy=(0.0, 0.0), xycoords="axes fraction", xytext=(0, -36),
                textcoords="offset points", fontsize=6.8, style="italic", color="#555555",
                va="top", ha="left")


# Human-friendly dataset display names (keep titles short so they do not truncate at figure width).
_DISP = {
    "tennessee_eastman_idv01": "Tennessee Eastman IDV(1)",
    "tennessee_eastman_idv04": "Tennessee Eastman IDV(4)",
    "tennessee_eastman_idv06": "Tennessee Eastman IDV(6)",
    "tennessee_eastman_idv13": "Tennessee Eastman IDV(13)",
    "tennessee_eastman_idv14": "Tennessee Eastman IDV(14)",
    "cstr_reactor": "CSTR reactor",
    "gas_sensor_array_drift": "Gas-sensor array drift",
    "penicillin_fedbatch": "Penicillin fed-batch",
    "bsm1_wastewater": "BSM1 wastewater",
    "three_tank": "Three-tank",
    "swat_water_treatment_standin": "SWaT (stand-in)",
}


def disp(name):
    """Return a short human-readable display name for a dataset (falls back to a tidied raw name)."""
    return _DISP.get(name, name.replace("_", " "))
