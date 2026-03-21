#!/usr/bin/env python3
"""Convert deterministic NASA public raw summaries into DSFB observed/predicted CSV pairs."""

from __future__ import annotations

import argparse
import csv
import json
from dataclasses import dataclass, asdict
from pathlib import Path


DATASETS = ("nasa_milling", "nasa_bearings")


@dataclass
class ProcessedMetadata:
    dataset: str
    raw_summary_csv: str
    observed_csv: str
    predicted_csv: str
    selected_channels: list[str]
    baseline_window: int
    trailing_prediction_window: int
    normalization_note: str


def crate_root() -> Path:
    return Path(__file__).resolve().parents[1]


def raw_summary_path(dataset: str) -> Path:
    return crate_root() / "data" / "public_dataset" / "raw" / f"{dataset}_raw_summary.csv"


def processed_dir(dataset: str) -> Path:
    return crate_root() / "data" / "processed" / dataset


def dataset_channels(dataset: str) -> tuple[list[str], str]:
    if dataset == "nasa_milling":
        return (
            ["vib_table_rms", "ae_spindle_rms", "smc_dc_rms"],
            "NASA Milling case 11 per-run RMS feature ratios relative to the early healthy baseline.",
        )
    if dataset == "nasa_bearings":
        return (
            ["bearing3_rms", "bearing4_rms", "bearing4_kurtosis"],
            "NASA IMS bearings set 1 last-64-file progression using failing-bearing RMS and kurtosis feature ratios.",
        )
    raise SystemExit(f"unsupported dataset {dataset}")


def dataset_time_key(dataset: str) -> str:
    return {
        "nasa_milling": "time_minutes",
        "nasa_bearings": "relative_hours",
    }[dataset]


def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def read_rows(path: Path) -> list[dict[str, str]]:
    with path.open("r", newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    ensure_dir(path.parent)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def preprocess_dataset(dataset: str) -> None:
    raw_path = raw_summary_path(dataset)
    if not raw_path.is_file():
        raise SystemExit(f"{dataset}: missing raw summary {raw_path}")
    rows = read_rows(raw_path)
    channels, note = dataset_channels(dataset)
    time_key = dataset_time_key(dataset)
    baseline_window = 4 if dataset == "nasa_bearings" else 5
    prediction_window = 4

    baselines = {}
    for channel in channels:
        values = [float(row[channel]) for row in rows[:baseline_window]]
        baselines[channel] = sum(values) / len(values)

    normalized_rows = []
    for row in rows:
        normalized = {channel: float(row[channel]) / baselines[channel] for channel in channels}
        normalized_rows.append(normalized)

    observed_rows = []
    predicted_rows = []
    for index, (raw_row, normalized) in enumerate(zip(rows, normalized_rows)):
        observed_row = {
            "step": index,
            "time": float(raw_row[time_key]),
        }
        predicted_row = {
            "step": index,
            "time": float(raw_row[time_key]),
        }
        for channel in channels:
            observed_row[channel] = normalized[channel]
            if index == 0:
                predicted_row[channel] = normalized[channel]
            else:
                start = max(0, index - prediction_window)
                history = [normalized_rows[position][channel] for position in range(start, index)]
                predicted_row[channel] = sum(history) / len(history)
        observed_rows.append(observed_row)
        predicted_rows.append(predicted_row)

    output_dir = processed_dir(dataset)
    observed_path = output_dir / "observed.csv"
    predicted_path = output_dir / "predicted.csv"
    metadata_path = output_dir / "metadata.json"
    write_rows(observed_path, observed_rows)
    write_rows(predicted_path, predicted_rows)
    ensure_dir(output_dir)
    metadata = ProcessedMetadata(
        dataset=dataset,
        raw_summary_csv=str(raw_path.relative_to(crate_root())),
        observed_csv=str(observed_path.relative_to(crate_root())),
        predicted_csv=str(predicted_path.relative_to(crate_root())),
        selected_channels=channels,
        baseline_window=baseline_window,
        trailing_prediction_window=prediction_window,
        normalization_note=note,
    )
    with metadata_path.open("w", encoding="utf-8") as handle:
        json.dump(asdict(metadata), handle, indent=2)
        handle.write("\n")
    print(f"{dataset}: wrote {observed_path} and {predicted_path}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Preprocess deterministic NASA raw-summary slices into DSFB CSV inputs"
    )
    parser.add_argument(
        "--dataset",
        action="append",
        choices=DATASETS,
        help="Dataset to preprocess. Defaults to both NASA demo datasets.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    for dataset in args.dataset or list(DATASETS):
        preprocess_dataset(dataset)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
