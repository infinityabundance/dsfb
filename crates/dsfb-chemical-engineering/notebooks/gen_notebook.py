#!/usr/bin/env python3
"""Generate the DSFB-Chemical-Engineering Colab reproducibility notebook.

The notebook (sections 1-8, modelled on the DSFB-GPU replay notebook) builds both crates, verifies
dataset SHA-256 provenance, runs the 20-dataset edge audit and the CUDA forensic court, packs all
artifacts into a downloadable ZIP, verifies byte-exact replay, and renders the publication figures.
It deliberately does NOT compile the paper.

Run:  python3 notebooks/gen_notebook.py  ->  notebooks/dsfb_chemical_engineering_colab.ipynb

Output: notebooks/dsfb_chemical_engineering_colab.ipynb
  A standard Jupyter notebook (nbformat 4) with 8 sections (markdown + code cell pairs).
  The notebook is designed to run on Google Colab with a GPU runtime; it also runs on
  any local Jupyter with the repo checked out.

  Notebook sections:
    §1. Environment report + source bundle  (prints Python/GPU/Rust versions; clones repo if needed)
    §2. Build both crates                   (edge always; CUDA crate only when nvcc is present)
    §3. Dataset SHA-256 verification        (provenance gate: re-fetches if slices absent)
    §4. Run 20-dataset edge audit + CUDA court
    §5. Pack artifact ZIP + Colab download  (google.colab.files.download if available)
    §6. Byte-exact replay + cross-backend verification
    §7. Result summary + publication figures (calls gen_figures.py; displays PNGs inline)
    §8. Non-claims and citation pointers

  The notebook cells are constructed as Python strings (not raw source files) so that
  gen_notebook.py is the single authoritative source — editing it regenerates the notebook.
  Cell source strings use \\n (literal newline) so json.dump produces a clean notebook.

Determinism: the notebook itself is deterministic (same string constants -> same JSON);
the pipeline it invokes is also deterministic given committed inputs.
"""
import json
import os

# Output path is determined relative to this script's location (notebooks/).
HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "dsfb_chemical_engineering_colab.ipynb")

# Colab badge rendered in the first markdown cell; links to the notebook on main branch.
BADGE = (
    "[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)]"
    "(https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)"
)


def md(text):
    """Return a Jupyter markdown cell dict with the given text as source."""
    return {"cell_type": "markdown", "metadata": {}, "source": text}


def code(text):
    """Return a Jupyter code cell dict with the given text as source and no prior outputs."""
    return {"cell_type": "code", "metadata": {}, "execution_count": None, "outputs": [], "source": text}


# `cells` is the ordered list of notebook cell dicts that will be serialised to JSON.
cells = []

cells.append(md(f"""# DSFB-Chemical-Engineering — reproducibility notebook

{BADGE}

**Read-only residual semiotics for chemometrics-augmented fault detection, with a deterministic,
byte-exact, CUDA-accelerated forensic evidence court.**
*Riaan de Beer — Invariant Forge LLC — ORCID 0009-0006-1155-027X.*

This notebook reproduces both crates end-to-end:
1. environment report + source bundle, 2. build both crates, 3. dataset SHA-256 verification,
4. run the 20-dataset edge audit + CUDA forensic court, 5. pack a downloadable artifact ZIP,
6. byte-exact replay + cross-backend verification, 7. result summary + figures, 8. non-claims.

It **shows code feedback and figures** and **does not compile the paper**. Click **Runtime → Run all**.
For the CUDA path, use a GPU runtime (Runtime → Change runtime type → GPU)."""))

cells.append(md("## §1. Environment report + source bundle"))
cells.append(code(
    'import os, sys, subprocess, shutil, pathlib\n'
    'def sh(c):\n'
    '    print("$", c)\n'
    '    print(subprocess.run(c, shell=True, capture_output=True, text=True).stdout[-4000:])\n'
    'print("python", sys.version.split()[0])\n'
    'sh("nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv 2>/dev/null || echo no-gpu")\n'
    'sh("nvcc --version 2>/dev/null | tail -2 || echo no-nvcc")\n'
    '# Rust toolchain (install if missing, e.g. on a fresh Colab runtime).\n'
    'if shutil.which("cargo") is None:\n'
    '    print("installing rust...")\n'
    '    subprocess.run("curl -sSf https://sh.rustup.rs | sh -s -- -y", shell=True)\n'
    '    os.environ["PATH"] = os.path.expanduser("~/.cargo/bin") + ":" + os.environ["PATH"]\n'
    'sh("cargo --version 2>/dev/null || echo no-cargo")\n'
))
cells.append(code(
    '# Locate or fetch the repository. Set DSFB_REPO_DIR to use a local checkout; otherwise clone.\n'
    'REPO_URL = os.environ.get("DSFB_REPO_URL", "https://github.com/infinityabundance/dsfb")\n'
    'CRATE_SUBDIR = "crates/dsfb-chemical-engineering"  # the crate lives under the dsfb monorepo\n'
    'REPO_DIR = os.environ.get("DSFB_REPO_DIR", "")\n'
    'if not REPO_DIR:\n'
    '    if pathlib.Path("Cargo.toml").exists() and pathlib.Path("crates/dsfb-chemical-engineering-edge").exists():\n'
    '        REPO_DIR = os.getcwd()\n'
    '    elif pathlib.Path("dsfb/" + CRATE_SUBDIR).exists():\n'
    '        REPO_DIR = str(pathlib.Path("dsfb/" + CRATE_SUBDIR).resolve())\n'
    '    else:\n'
    '        print("cloning", REPO_URL)\n'
    '        subprocess.run(f"git clone --depth 1 {REPO_URL}", shell=True)\n'
    '        REPO_DIR = str(pathlib.Path("dsfb/" + CRATE_SUBDIR).resolve())\n'
    'os.chdir(REPO_DIR)\n'
    'os.environ["DSFB_REPO_DIR"] = REPO_DIR\n'
    'if pathlib.Path("/opt/cuda/bin").exists():\n'
    '    os.environ["PATH"] = "/opt/cuda/bin:" + os.environ["PATH"]; os.environ["CUDA_HOME"] = "/opt/cuda"\n'
    'print("REPO_DIR:", REPO_DIR)\n'
    'sh("ls -1")\n'
))

cells.append(md("## §2. Build both crates (edge always; CUDA if a GPU + nvcc are present)"))
cells.append(code(
    '%%bash\n'
    'cd "$DSFB_REPO_DIR"\n'
    'set -e\n'
    'echo "=== building edge crate ==="\n'
    'cargo build --release -p dsfb-chemical-engineering-edge 2>&1 | tail -3\n'
    'echo "=== building cuda crate ==="\n'
    'bash crates/dsfb-chemical-engineering-cuda/scripts/build_cuda.sh 2>&1 | tail -4\n'
))

cells.append(md("## §3. Dataset SHA-256 verification (provenance gate)\n"
                 "Fetch the dataset slices if absent, then confirm each committed slice matches the SHA-256 in the provenance manifest."))
cells.append(code(
    'import hashlib, re\n'
    'edge = pathlib.Path("crates/dsfb-chemical-engineering-edge")\n'
    'slices = edge / "data" / "slices"\n'
    'if not slices.exists() or not any(slices.glob("*.csv")):\n'
    '    sh(f"python3 {edge}/scripts/fetch_datasets.py")\n'
    'man = (edge / "data" / "MANIFEST.toml").read_text()\n'
    'blocks = re.findall(r"\\[\\[dataset\\]\\](.*?)(?=\\[\\[dataset\\]\\]|\\Z)", man, re.S)\n'
    'ok = bad = 0\n'
    'for b in blocks:\n'
    '    name = re.search(r\'name = "([^"]+)"\', b); sha = re.search(r\'sha256 = "([^"]+)"\', b)\n'
    '    if not name or not sha: continue\n'
    '    p = slices / f"{name.group(1)}.csv"\n'
    '    if not p.exists(): print("MISSING", p.name); bad += 1; continue\n'
    '    h = hashlib.sha256(p.read_bytes()).hexdigest()\n'
    '    if h == sha.group(1): ok += 1\n'
    '    else: print("SHA MISMATCH", p.name); bad += 1\n'
    'print(f"dataset SHA-256 gate: {ok} verified, {bad} failed")\n'
    'assert bad == 0, "dataset provenance gate failed"\n'
))

cells.append(md("## §4. Run the 20-dataset edge audit + CUDA forensic court"))
cells.append(code(
    '%%bash\n'
    'cd "$DSFB_REPO_DIR"\n'
    'export DSFB_CHEM_DIR="$DSFB_REPO_DIR/crates/dsfb-chemical-engineering-edge"\n'
    'export DSFB_CHEM_CUDA_DIR="$DSFB_REPO_DIR/crates/dsfb-chemical-engineering-cuda"\n'
    'echo "=== EDGE DEMO ==="\n'
    './target/release/dsfb-chem-edge demo 2>&1 | tail -24\n'
    'echo ""; echo "=== CUDA FORENSIC COURT ==="\n'
    './target/release/dsfb-chem-cuda demo 2>&1 | tail -24\n'
))

cells.append(md("## §5. Pack all artifacts into a downloadable ZIP"))
cells.append(code(
    'import glob, datetime\n'
    'stamp = datetime.datetime.utcnow().strftime("%Y%m%d-%H%M%S")\n'
    'bundle = f"dsfb_chemical_engineering_artifacts_{stamp}"\n'
    'os.makedirs(bundle, exist_ok=True)\n'
    'def latest(p):\n'
    '    xs = sorted(glob.glob(p)); return xs[-1] if xs else None\n'
    'for src, dst in [(latest("output-dsfb-chemical-engineering/*"), "edge"),\n'
    '                 (latest("output-dsfb-chemical-engineering-cuda/*"), "cuda")]:\n'
    '    if src: shutil.copytree(src, f"{bundle}/{dst}", dirs_exist_ok=True)\n'
    'shutil.copytree("crates/dsfb-chemical-engineering-cuda/reports", f"{bundle}/nsight_reports", dirs_exist_ok=True)\n'
    'zip_path = shutil.make_archive(bundle, "zip", bundle)\n'
    'print("artifact bundle:", zip_path, f"({os.path.getsize(zip_path)//1024} KB)")\n'
    'try:\n'
    '    from google.colab import files; files.download(zip_path)\n'
    'except Exception as e:\n'
    '    print("(not in Colab; bundle saved locally)")\n'
))

cells.append(md("## §6. Byte-exact replay + cross-backend verification\n"
                 "Every dataset's CUDA evidence root must replay and match the CPU reference byte-for-byte."))
cells.append(code(
    '%%bash\n'
    'cd "$DSFB_REPO_DIR"\n'
    'export DSFB_CHEM_DIR="$DSFB_REPO_DIR/crates/dsfb-chemical-engineering-edge"\n'
    'export DSFB_CHEM_CUDA_DIR="$DSFB_REPO_DIR/crates/dsfb-chemical-engineering-cuda"\n'
    'echo "=== edge deterministic replay ==="\n'
    './target/release/dsfb-chem-edge verify-replay 2>&1 | tail -8\n'
    'echo "=== cuda forensic replay + cross-backend verification ==="\n'
    './target/release/dsfb-chem-cuda verify-replay 2>&1 | tail -24\n'
))

cells.append(md("## §7. Result summary + publication figures"))
cells.append(code(
    'sh("python3 crates/dsfb-chemical-engineering-edge/scripts/gen_figures.py")\n'
    'm = latest("output-dsfb-chemical-engineering/*/metrics.csv")\n'
    'if m: print(open(m).read())\n'
))
cells.append(code(
    'from IPython.display import Image, display\n'
    'for f in ["residual_timeline_tennessee_eastman_idv01.png", "detection_delay.png",\n'
    '          "episode_compression.png", "atlas_families.png"]:\n'
    '    p = pathlib.Path("paper/figures") / f\n'
    '    if p.exists(): display(Image(str(p)))\n'
))

cells.append(md("""## §8. Non-claims and citation pointers

**This notebook does not compile the paper.** Build the PDF separately with `bash paper/build_paper.sh`.

**Non-claims.** DSFB-Chemical-Engineering is augmentation, not competition: it makes **no** claim of
higher accuracy or faster detection than established chemometrics; residual motifs are structural
candidates, not proven root causes; public simulation benchmarks are not plants; and the framework
carries no safety or control authority. The correct failure mode is to emit *"unknown structural
episode (evidence preserved)"* rather than to force a diagnosis.

**Cite:** see `CITATION.cff`. Author: Riaan de Beer, Invariant Forge LLC (ORCID 0009-0006-1155-027X).
Dataset provenance, licenses, and SHA-256 digests: `crates/dsfb-chemical-engineering-edge/data/MANIFEST.toml`."""))

# Assemble the nbformat 4 notebook dict.  `accelerator: GPU` hints Colab to offer a GPU runtime.
nb = {
    "cells": cells,
    "metadata": {
        "kernelspec": {"display_name": "Python 3", "language": "python", "name": "python3"},
        "language_info": {"name": "python"},
        "colab": {"provenance": [], "toc_visible": True},
        "accelerator": "GPU",
    },
    "nbformat": 4,
    "nbformat_minor": 5,
}

# Write with indent=1 to match the Jupyter/Colab conventional compact indentation.
with open(OUT, "w") as f:
    json.dump(nb, f, indent=1)
print("wrote", OUT, f"({len(cells)} cells)")
