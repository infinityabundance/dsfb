#!/usr/bin/env python3
"""
DSFB-RF Unified All-Figures Generator
======================================
Single-command pipeline:

  1. cargo run --features std,serde --example generate_figures_all
         → writes  dsfb-rf-output/figure_data_all.json  (all 51 figures, engine-backed)

  2. python3 figures_all.py --data dsfb-rf-output/figure_data_all.json ...
         → renders fig_01 … fig_40  (39 PDFs + 39 PNGs)

  3. pdfunite  → combined multi-page PDF
  4. zip       → artefact archive

Output tree
-----------
<workspace>/dsfb-rf-output/
  dsfb-rf-<YYYY-MM-DD_HH-MM-SS>/
    figs/
      fig_01_*.pdf  fig_01_*.png
      ...
      fig_40_*.pdf  fig_40_*.png
    dsfb-rf-all-figures_<ts>.pdf
    dsfb-rf-artifacts_<ts>.zip

Usage
-----
    python3 scripts/generate_all.py                  # full pipeline (cargo + python)
    python3 generate_all.py --skip-cargo     # skip Rust build (reuse existing JSON)
    python3 generate_all.py --dpi 300        # print-resolution output
    python3 generate_all.py --fig 5 21 37    # specific figures only
"""

import argparse
import subprocess
import sys
import zipfile
from datetime import datetime
from pathlib import Path


# ─── paths ─────────────────────────────────────────────────────────────────
SCRIPTS_DIR = Path(__file__).parent.resolve()          # …/dsfb-rf/dsfb-rf/scripts
CRATE_DIR   = SCRIPTS_DIR.parent.resolve()             # …/dsfb-rf/dsfb-rf
REPO_ROOT   = CRATE_DIR.parent.resolve()               # …/dsfb-rf  (workspace root)
OUTPUT_ROOT = REPO_ROOT / "dsfb-rf-output"


def die(msg: str):
    print(f"\n[ERROR] {msg}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], *, cwd: Path):
    """Run a subprocess; stream stdout; raise on failure."""
    print(f"  $ {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        die(f"Command failed (exit {result.returncode}): {' '.join(cmd)}")


def merge_pdfs(pdf_files: list[Path], dest: Path):
    """Merge sorted list of PDFs into a single file using pdfunite."""
    if not pdf_files:
        print("  [warn] no PDFs to merge — skipping combined PDF")
        return
    sorted_pdfs = sorted(pdf_files, key=lambda p: p.name)
    cmd = ["pdfunite"] + [str(p) for p in sorted_pdfs] + [str(dest)]
    print(f"\n  Merging {len(sorted_pdfs)} PDFs → {dest.name}")
    result = subprocess.run(cmd)
    if result.returncode != 0:
        print(f"  [warn] pdfunite failed (exit {result.returncode}) — combined PDF not created")
    else:
        size_mb = dest.stat().st_size / 1_048_576
        print(f"  Combined PDF: {dest.name}  ({size_mb:.1f} MB)")


def make_zip(run_dir: Path, zip_path: Path):
    """Zip the entire run_dir (excluding the zip file itself)."""
    print(f"\n  Creating archive → {zip_path.name}")
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for path in sorted(run_dir.rglob("*")):
            if path == zip_path:
                continue
            arcname = path.relative_to(run_dir.parent)  # keep run dir name in archive
            zf.write(path, arcname)
    size_mb = zip_path.stat().st_size / 1_048_576
    print(f"  Archive:      {zip_path.name}  ({size_mb:.1f} MB)")


def main():
    parser = argparse.ArgumentParser(
        description="Generate all 40 DSFB-RF publication figures into a timestamped folder"
    )
    parser.add_argument("--dpi",        type=int, default=150,
                        help="Output resolution in DPI (default: 150; use 300 for print)")
    parser.add_argument("--skip-cargo", action="store_true",
                        help="Skip 'cargo run' step; reuse existing figure_data_all.json")
    parser.add_argument("--out-root",   type=str, default=None,
                        help=f"Override output root (default: {OUTPUT_ROOT})")
    parser.add_argument("--fig",        type=int, nargs="*",
                        help="Render specific figure numbers only (default: all)")
    args = parser.parse_args()

    # ── timestamped run directory ──────────────────────────────────────────
    ts       = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    run_name = f"dsfb-rf-{ts}"
    out_root = Path(args.out_root) if args.out_root else OUTPUT_ROOT
    run_dir  = out_root / run_name
    figs_dir = run_dir / "figs"
    figs_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 64)
    print(f"  DSFB-RF Unified Figure Generator   {ts}")
    print(f"  Output folder: {run_dir}")
    print("=" * 64)

    python    = sys.executable
    crate_dir = CRATE_DIR
    data_json = OUTPUT_ROOT / "figure_data_all.json"           # written by cargo run

    # ── Step 1: Rust data generator ────────────────────────────────────────
    if args.skip_cargo:
        print("\n  [skip] cargo build (--skip-cargo)")
        if not data_json.exists():
            die(f"{data_json.name} not found — cannot skip cargo step")
    else:
        print("\n--- Step 1: cargo run (generate_figures_all) ---")
        run(
            ["cargo", "run", "--release",
             "--features", "std,serde",
             "--example", "generate_figures_all"],
            cwd=crate_dir,
        )
        if not data_json.exists():
            die(f"Expected {data_json} after cargo run — not found")
        print(f"  JSON ready: {data_json.name}  "
              f"({data_json.stat().st_size / 1024:.0f} KB)")

    # ── Step 2: Python renderer ────────────────────────────────────────────
    print(f"\n--- Step 2: figures_all.py (fig_01 – fig_40) ---")
    fig_cmd = [
        python, str(SCRIPTS_DIR / "figures_all.py"),
        "--data", str(data_json),
        "--out",  str(figs_dir),
        "--dpi",  str(args.dpi),
    ]
    if args.fig:
        fig_cmd += ["--fig"] + [str(f) for f in args.fig]
    run(fig_cmd, cwd=REPO_ROOT)  # run from repo root so relative paths resolve

    # ── Combined PDF ───────────────────────────────────────────────────────
    all_pdfs = sorted(figs_dir.glob("fig_*.pdf"))
    combined_pdf = run_dir / f"dsfb-rf-all-figures_{ts}.pdf"
    merge_pdfs(all_pdfs, combined_pdf)

    # ── ZIP archive ────────────────────────────────────────────────────────
    zip_path = run_dir / f"dsfb-rf-artifacts_{ts}.zip"
    make_zip(run_dir, zip_path)

    # ── summary ───────────────────────────────────────────────────────────
    fig_count  = len(list(figs_dir.glob("fig_*.png")))
    total_size = sum(p.stat().st_size for p in run_dir.rglob("*") if p.is_file()) / 1_048_576
    print()
    print("=" * 64)
    print(f"  Figures generated : {fig_count}")
    print(f"  Run folder        : {run_dir}")
    print(f"  Combined PDF      : {combined_pdf.name}")
    print(f"  ZIP archive       : {zip_path.name}")
    print(f"  Total on-disk     : {total_size:.1f} MB")
    print("=" * 64)


if __name__ == "__main__":
    main()
