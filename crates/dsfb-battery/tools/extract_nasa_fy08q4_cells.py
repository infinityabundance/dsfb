#!/usr/bin/env python3
"""
Extract capacity-per-cycle CSVs for the FY08Q4 NASA PCoE battery cells.

This helper is additive to the existing B0005-only extractor. It reads the
already-downloaded FY08Q4 MATLAB files and writes CSVs for the cell-level
repeat-evaluation helpers:
  - B0005
  - B0006
  - B0007
  - B0018

Output files:
  data/nasa_b0005_capacity.csv
  data/nasa_b0006_capacity.csv
  data/nasa_b0007_capacity.csv
  data/nasa_b0018_capacity.csv
"""

import csv
import os
import sys

try:
    from scipy.io import loadmat
except ImportError:
    print("ERROR: scipy is required. Install with: pip install scipy")
    sys.exit(1)


CELL_IDS = ["B0005", "B0006", "B0007", "B0018"]


def extract_cell_capacity(mat_path: str, cell_id: str) -> list[tuple[int, float]]:
    data = loadmat(mat_path, simplify_cells=True)
    cell = data[cell_id]
    cycles = cell["cycle"]
    discharge = []
    cycle_num = 0
    for item in cycles:
        if str(item["type"]).strip() != "discharge":
            continue
        cycle_num += 1
        cap_data = item["data"]
        if "Capacity" in cap_data:
            discharge.append((cycle_num, float(cap_data["Capacity"])))
    return discharge


def write_csv(output_csv: str, rows: list[tuple[int, float]]) -> None:
    os.makedirs(os.path.dirname(output_csv) or ".", exist_ok=True)
    with open(output_csv, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["cycle", "capacity_ah", "type"])
        for cycle, capacity in rows:
            writer.writerow([cycle, f"{capacity:.6f}", "discharge"])


if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    base_dir = os.path.dirname(script_dir)
    mat_dir = os.path.join(base_dir, "data", "tmp_extract", "fy08")
    output_dir = os.path.join(base_dir, "data")

    missing = []
    for cell_id in CELL_IDS:
        mat_path = os.path.join(mat_dir, f"{cell_id}.mat")
        if not os.path.exists(mat_path):
            missing.append(mat_path)
            continue

        rows = extract_cell_capacity(mat_path, cell_id)
        output_csv = os.path.join(output_dir, f"nasa_{cell_id.lower()}_capacity.csv")
        write_csv(output_csv, rows)
        print(f"Wrote {output_csv} ({len(rows)} discharge cycles)")

    if missing:
        print("Missing MAT files:")
        for path in missing:
            print(f"  - {path}")
        sys.exit(1)
