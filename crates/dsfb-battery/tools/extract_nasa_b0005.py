#!/usr/bin/env python3
"""
Extract capacity-per-cycle data from the NASA PCoE Battery Dataset (B0005.mat).

This script reads the official NASA Prognostics Center of Excellence battery
aging dataset (.mat format) and extracts the discharge capacity for each cycle
of cell B0005 (18650 Li-ion, constant-current discharge at 2A, 24°C ambient).

Data provenance:
    NASA Ames Prognostics Center of Excellence
    https://www.nasa.gov/content/prognostics-center-of-excellence-data-set-repository
    https://phm-datasets.s3.amazonaws.com/NASA/5.+Battery+Data+Set.zip

    B. Saha and K. Goebel (2007). "Battery Data Set",
    NASA Ames Prognostics Data Repository, NASA Ames Research Center,
    Moffett Field, CA.

Output: data/nasa_b0005_capacity.csv
    Columns: cycle, capacity_ah, type
    - cycle: discharge cycle number (1-indexed)
    - capacity_ah: measured discharge capacity in Ah
    - type: "discharge" for all rows
"""

import os
import sys
import csv
import numpy as np

try:
    from scipy.io import loadmat
except ImportError:
    print("ERROR: scipy is required. Install with: pip install scipy")
    sys.exit(1)


def extract_b0005_capacity(mat_path: str, output_csv: str) -> None:
    """Extract per-cycle discharge capacity from B0005.mat and write to CSV."""
    print(f"Loading {mat_path} ...")
    data = loadmat(mat_path, simplify_cells=True)

    # The .mat file contains a struct array 'B0005' with one entry per cycle.
    # Each entry has fields: 'cycle', 'type', 'ambient_temperature', 'time', 'data'.
    # 'data' contains measurement arrays; for discharge cycles, it includes
    # 'Capacity' which is the measured discharge capacity.
    b0005 = data["B0005"]

    # b0005 is a structured numpy array. Each element is a cycle struct.
    # Access the 'cycle' array which contains cycle structs.
    cycles_struct = b0005["cycle"]

    # cycles_struct is an array of structs. Each struct has:
    #   type: 'charge', 'discharge', or 'impedance'
    #   ambient_temperature: float
    #   time: datetime string
    #   data: struct with measurement arrays

    discharge_capacities = []
    cycle_num = 0

    for i in range(len(cycles_struct)):
        entry = cycles_struct[i]
        cycle_type = str(entry["type"]).strip()

        if cycle_type == "discharge":
            cycle_num += 1
            cap_data = entry["data"]
            # With simplify_cells=True, data is a dict.
            # Capacity is a scalar float: the measured discharge capacity in Ah.
            if "Capacity" in cap_data:
                capacity = float(cap_data["Capacity"])
                discharge_capacities.append((cycle_num, capacity))

    print(f"Extracted {len(discharge_capacities)} discharge cycles from B0005")

    # Write CSV
    os.makedirs(os.path.dirname(output_csv) or ".", exist_ok=True)
    with open(output_csv, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["cycle", "capacity_ah", "type"])
        for cyc, cap in discharge_capacities:
            writer.writerow([cyc, f"{cap:.6f}", "discharge"])

    print(f"Written to {output_csv}")
    print(f"  Cycles: {len(discharge_capacities)}")
    if discharge_capacities:
        print(f"  Initial capacity: {discharge_capacities[0][1]:.4f} Ah")
        print(f"  Final capacity:   {discharge_capacities[-1][1]:.4f} Ah")


if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    base_dir = os.path.dirname(script_dir)  # crates/dsfb-battery/
    mat_path = os.path.join(base_dir, "data", "tmp_extract", "fy08", "B0005.mat")
    output_csv = os.path.join(base_dir, "data", "nasa_b0005_capacity.csv")

    if not os.path.exists(mat_path):
        print(f"ERROR: {mat_path} not found.")
        print("Run the dataset download first:")
        print("  curl -L -o data/nasa_battery_dataset.zip \\")
        print('    "https://phm-datasets.s3.amazonaws.com/NASA/5.+Battery+Data+Set.zip"')
        sys.exit(1)

    extract_b0005_capacity(mat_path, output_csv)
