#!/usr/bin/env python3
"""
build_output.py — Full DSFB-RF output pipeline.

Runs cargo examples, renders all figures, merges into combined PDF,
copies JSON artifacts, and zips everything into a single archive.

Output folder: ../dsfb-rf-output/dsfb-rf-<YYYY-MM-DD_HH-MM-SS>/
  figs/                 — 50 individual figure PDFs + PNGs
  dsfb-rf-all-figures.pdf  — all figures merged into one PDF
  figure_data.json      — Phase-1 engine data
  figure_data_all.json  — all-phases engine data
  dsfb-rf-<timestamp>-artifacts.zip  — complete zip of above

Usage:
  cd dsfb-rf/
  python3 scripts/build_output.py
"""

import datetime
import glob
import os
import subprocess
import sys
import zipfile

CRATE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUTPUT_ROOT = os.path.join(CRATE_DIR, "..", "dsfb-rf-output")
FIGURES_ALL_PY = os.path.join(CRATE_DIR, "scripts", "figures_all.py")


def run(cmd, **kwargs):
    print(f"  $ {' '.join(cmd)}")
    subprocess.run(cmd, check=True, cwd=CRATE_DIR, **kwargs)


def main():
    ts = datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    out_name = f"dsfb-rf-{ts}"
    out_dir = os.path.normpath(os.path.join(OUTPUT_ROOT, out_name))
    figs_dir = os.path.join(out_dir, "figs")
    data_dir = os.path.normpath(OUTPUT_ROOT)

    os.makedirs(figs_dir, exist_ok=True)
    os.makedirs(data_dir, exist_ok=True)

    print(f"\n{'='*60}")
    print(f"  DSFB-RF build_output.py")
    print(f"  Output folder: {out_dir}")
    print(f"{'='*60}\n")

    # 1. Generate Phase-1 data
    print("[1/5] Generating Phase-1 figure data (generate_figures)...")
    run(["cargo", "run", "--example", "generate_figures", "--features", "std,serde"])

    # 2. Generate all-phases data
    print("\n[2/5] Generating all-phases figure data (generate_figures_all)...")
    run(["cargo", "run", "--example", "generate_figures_all", "--features", "std,serde"])

    # 3. Render figures
    print(f"\n[3/5] Rendering figures → {figs_dir}")
    json_all = os.path.join(data_dir, "figure_data_all.json")
    run(["python3", FIGURES_ALL_PY, "--data", json_all, "--out", figs_dir])

    # 4. Copy JSON artifacts into output folder
    print("\n[4/5] Copying JSON artifacts...")
    for j in ["figure_data.json", "figure_data_all.json"]:
        src = os.path.join(data_dir, j)
        dst = os.path.join(out_dir, j)
        if os.path.exists(src):
            import shutil
            shutil.copy2(src, dst)
            print(f"  Copied {j}")

    # 5. Merge PDFs + zip
    print("\n[5/5] Merging PDFs and creating artifact zip...")
    pdfs = sorted(glob.glob(os.path.join(figs_dir, "*.pdf")))
    combined_pdf = os.path.join(out_dir, "dsfb-rf-all-figures.pdf")
    subprocess.run(["pdfunite"] + pdfs + [combined_pdf], check=True)
    print(f"  Combined PDF: dsfb-rf-all-figures.pdf ({os.path.getsize(combined_pdf)//1024} KB)")

    zip_path = os.path.join(out_dir, f"{out_name}-artifacts.zip")
    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zf:
        for f in sorted(glob.glob(os.path.join(figs_dir, "*"))):
            zf.write(f, os.path.join("figs", os.path.basename(f)))
        zf.write(combined_pdf, "dsfb-rf-all-figures.pdf")
        for j in ["figure_data.json", "figure_data_all.json"]:
            src = os.path.join(out_dir, j)
            if os.path.exists(src):
                zf.write(src, j)
    print(f"  Zip: {out_name}-artifacts.zip ({os.path.getsize(zip_path)//1024} KB)")

    print(f"\n{'='*60}")
    print(f"  Done. All artifacts in:")
    print(f"  dsfb-rf-output/{out_name}/")
    print(f"{'='*60}\n")


if __name__ == "__main__":
    main()
