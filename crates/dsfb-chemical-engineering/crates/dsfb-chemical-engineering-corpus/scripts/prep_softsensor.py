#!/usr/bin/env python3
"""Seal the provenance of a locally-held soft-sensor dataset WITHOUT redistributing it.

This crate catalogues public soft-sensor datasets (see src/seed.rs) but never vendors their bytes.
When you hold a local copy (e.g. the Quality-Prediction-in-a-Mining-Process flotation set the user
placed in research/), this utility emits a deterministic provenance seal: the SHA-256 of the dataset
file plus its row/column shape. The seal (git-ignored) lets a deterministic witness later reference an
exact dataset version; the data itself stays local and is never copied into the repository.

Default target: the mining-flotation set. Override with env DATASET_PATH (a .csv or .zip).

Run:  python3 scripts/prep_softsensor.py
"""
import csv
import hashlib
import io
import json
import os
import zipfile

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT_DIR = os.path.join(HERE, "data")  # git-ignored
os.makedirs(OUT_DIR, exist_ok=True)

DEFAULT = os.path.join(
    HERE, "..", "..", "research", "Quality Prediction in a Mining Process.zip"
)
DATASET_PATH = os.environ.get("DATASET_PATH", os.path.abspath(DEFAULT))
DATASET_ID = os.environ.get("DATASET_ID", "mining_flotation")


def read_csv_bytes(path):
    """Return (csv_bytes, member_name) for a .csv or the first .csv inside a .zip."""
    if path.endswith(".zip"):
        with zipfile.ZipFile(path) as z:
            members = [n for n in z.namelist() if n.lower().endswith(".csv")]
            if not members:
                raise SystemExit(f"no CSV inside {path}")
            name = members[0]
            return z.read(name), name
    with open(path, "rb") as f:
        return f.read(), os.path.basename(path)


def main():
    if not os.path.exists(DATASET_PATH):
        raise SystemExit(
            f"dataset not found: {DATASET_PATH}\n"
            f"Set DATASET_PATH to your local copy (this script never downloads or redistributes data)."
        )
    raw, member = read_csv_bytes(DATASET_PATH)
    sha = hashlib.sha256(raw).hexdigest()

    # Shape (rows x cols) without holding the whole thing in memory twice.
    reader = csv.reader(io.StringIO(raw.decode("utf-8", errors="replace")))
    header = next(reader, [])
    n_cols = len(header)
    n_rows = sum(1 for _ in reader)

    seal = {
        "dataset_id": DATASET_ID,
        "member": member,
        "sha256": sha,
        "n_rows": n_rows,
        "n_cols": n_cols,
        "note": "provenance seal only; dataset bytes are NOT redistributed by this repository",
    }
    out = os.path.join(OUT_DIR, f"{DATASET_ID}.provenance.json")
    with open(out, "w") as f:
        json.dump(seal, f, indent=2)
    print(f"sealed {DATASET_ID}: sha256={sha}  shape={n_rows}x{n_cols}  (member={member})")
    print(f"wrote {out} (git-ignored; not redistributed)")


if __name__ == "__main__":
    main()
