#!/usr/bin/env python3
"""Fetch and deterministically summarize the NASA public dataset demos."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import math
import shutil
import subprocess
import sys
import tempfile
import urllib.request
import zipfile
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path

import numpy as np
from scipy.io import loadmat


DATASETS = ("nasa_milling", "nasa_bearings")


@dataclass
class DatasetSourceMetadata:
    dataset: str
    source_url: str
    source_archive: str
    source_sha256: str
    raw_summary_csv: str
    record_count: int
    selection_note: str


def crate_root() -> Path:
    return Path(__file__).resolve().parents[1]


def source_root() -> Path:
    return crate_root() / "data" / "public_dataset" / "source"


def raw_root() -> Path:
    return crate_root() / "data" / "public_dataset" / "raw"


def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def dataset_archive_path(dataset: str) -> Path:
    return source_root() / {
        "nasa_milling": "nasa_milling.zip",
        "nasa_bearings": "nasa_bearings.zip",
    }[dataset]


def dataset_raw_summary_path(dataset: str) -> Path:
    return raw_root() / f"{dataset}_raw_summary.csv"


def dataset_metadata_path(dataset: str) -> Path:
    return raw_root() / f"{dataset}_source_metadata.json"


def dataset_source_url(dataset: str) -> str:
    return {
        "nasa_milling": "https://phm-datasets.s3.amazonaws.com/NASA/3.+Milling.zip",
        "nasa_bearings": "https://phm-datasets.s3.amazonaws.com/NASA/4.+Bearings.zip",
    }[dataset]


def dataset_source_sha256(dataset: str) -> str:
    return {
        "nasa_milling": "bdba8d52ec1a1baab24c2be58480e6ac62508c8cc1f8219f47ebde8fc9ebc474",
        "nasa_bearings": "21001ac266c465f5d345ec42d7b508c6a6328487fd9d4d7774422dd5ea10ad83",
    }[dataset]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def maybe_download_archive(dataset: str, force_download: bool) -> Path:
    ensure_dir(source_root())
    archive_path = dataset_archive_path(dataset)
    expected_sha = dataset_source_sha256(dataset)
    if archive_path.is_file() and not force_download:
        actual = sha256_file(archive_path)
        if actual == expected_sha:
            print(f"{dataset}: source archive already present at {archive_path}")
            return archive_path
        archive_path.unlink()

    with tempfile.TemporaryDirectory(prefix=f"dsfb-{dataset}-download-") as temp_dir:
        temp_path = Path(temp_dir) / archive_path.name
        print(f"{dataset}: downloading {dataset_source_url(dataset)}")
        with urllib.request.urlopen(dataset_source_url(dataset)) as response:
            with temp_path.open("wb") as handle:
                shutil.copyfileobj(response, handle)
        actual = sha256_file(temp_path)
        if actual != expected_sha:
            raise SystemExit(
                f"{dataset}: checksum mismatch for {temp_path}: expected {expected_sha}, got {actual}"
            )
        shutil.move(str(temp_path), archive_path)
    return archive_path


def existing_raw_summary_is_usable(dataset: str) -> bool:
    raw_path = dataset_raw_summary_path(dataset)
    metadata_path = dataset_metadata_path(dataset)
    return raw_path.is_file() and metadata_path.is_file()


def run_command(args: list[str]) -> str:
    return subprocess.check_output(args, text=True)


def rms(values: np.ndarray) -> float:
    values = np.asarray(values, dtype=np.float64)
    return float(np.sqrt(np.mean(np.square(values))))


def kurtosis(values: np.ndarray) -> float:
    values = np.asarray(values, dtype=np.float64)
    centered = values - np.mean(values)
    variance = np.mean(np.square(centered))
    if not math.isfinite(variance) or variance <= 0.0:
        return 0.0
    return float(np.mean(np.power(centered, 4.0)) / (variance * variance))


def write_metadata(metadata: DatasetSourceMetadata) -> None:
    metadata_path = dataset_metadata_path(metadata.dataset)
    ensure_dir(metadata_path.parent)
    with metadata_path.open("w", encoding="utf-8") as handle:
        json.dump(asdict(metadata), handle, indent=2)
        handle.write("\n")


def extract_milling_raw_summary(force_regenerate: bool) -> None:
    dataset = "nasa_milling"
    raw_summary = dataset_raw_summary_path(dataset)
    if raw_summary.is_file() and not force_regenerate:
        print(f"{dataset}: raw summary already present at {raw_summary}")
        return

    archive_path = maybe_download_archive(dataset, force_download=False)
    with zipfile.ZipFile(archive_path) as outer_zip:
        inner_bytes = outer_zip.read("3. Milling/mill.zip")
    with zipfile.ZipFile(io.BytesIO(inner_bytes)) as inner_zip:
        mat_bytes = inner_zip.read("mill.mat")
    mat = loadmat(io.BytesIO(mat_bytes), squeeze_me=True, struct_as_record=False)

    rows = []
    for row in sorted(mat["mill"], key=lambda item: (int(item.case), int(item.run))):
        if int(row.case) != 11:
            continue
        vib_table_rms = rms(row.vib_table)
        vib_spindle_rms = rms(row.vib_spindle)
        ae_table_rms = rms(row.AE_table)
        ae_spindle_rms = rms(row.AE_spindle)
        smc_dc_rms = rms(row.smcDC)
        smc_ac_rms = rms(row.smcAC)
        values = [
            vib_table_rms,
            vib_spindle_rms,
            ae_table_rms,
            ae_spindle_rms,
            smc_dc_rms,
            smc_ac_rms,
        ]
        if not all(math.isfinite(value) for value in values):
            continue
        rows.append(
            {
                "step": len(rows),
                "case": int(row.case),
                "run": int(row.run),
                "time_minutes": int(row.time),
                "vb": float(row.VB),
                "doc": float(row.DOC),
                "feed": float(row.feed),
                "material": int(row.material),
                "vib_table_rms": vib_table_rms,
                "vib_spindle_rms": vib_spindle_rms,
                "ae_table_rms": ae_table_rms,
                "ae_spindle_rms": ae_spindle_rms,
                "smc_dc_rms": smc_dc_rms,
                "smc_ac_rms": smc_ac_rms,
            }
        )

    ensure_dir(raw_summary.parent)
    with raw_summary.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)

    write_metadata(
        DatasetSourceMetadata(
            dataset=dataset,
            source_url=dataset_source_url(dataset),
            source_archive=str(archive_path.relative_to(crate_root())),
            source_sha256=dataset_source_sha256(dataset),
            raw_summary_csv=str(raw_summary.relative_to(crate_root())),
            record_count=len(rows),
            selection_note=(
                "NASA Milling case 11 only. Each retained record is one machining run collapsed "
                "into deterministic RMS features for vibration, AE, and spindle-current traces."
            ),
        )
    )
    print(f"{dataset}: wrote raw summary {raw_summary}")


def ensure_bearing_intermediate_archives(archive_path: Path) -> tuple[Path, Path]:
    ims_7z = source_root() / "IMS.7z"
    if not ims_7z.is_file():
        with zipfile.ZipFile(archive_path) as outer_zip:
            with outer_zip.open("4. Bearings/IMS.7z") as source, ims_7z.open("wb") as target:
                shutil.copyfileobj(source, target)
    first_test_rar = source_root() / "1st_test.rar"
    if not first_test_rar.is_file():
        subprocess.run(
            [
                "7z",
                "e",
                "-y",
                f"-o{source_root()}",
                str(ims_7z),
                "1st_test.rar",
            ],
            check=True,
            stdout=subprocess.DEVNULL,
        )
    return ims_7z, first_test_rar


def extract_bearings_raw_summary(force_regenerate: bool) -> None:
    dataset = "nasa_bearings"
    raw_summary = dataset_raw_summary_path(dataset)
    if raw_summary.is_file() and not force_regenerate:
        print(f"{dataset}: raw summary already present at {raw_summary}")
        return

    archive_path = maybe_download_archive(dataset, force_download=False)
    _ims_7z, first_test_rar = ensure_bearing_intermediate_archives(archive_path)

    listing = run_command(["7z", "l", "-ba", str(first_test_rar)])
    files = []
    for line in listing.splitlines():
        if not line.strip() or " D...A " in line:
            continue
        parts = line.split()
        if not parts:
            continue
        relative_path = parts[-1]
        if relative_path.startswith("1st_test/"):
            files.append(relative_path)
    files = sorted(files)
    selected = files[-64:]
    if len(selected) != 64:
        raise SystemExit(f"{dataset}: expected 64 selected files, got {len(selected)}")

    rows = []
    with tempfile.TemporaryDirectory(prefix="dsfb-nasa-bearings-") as temp_dir:
        temp_root = Path(temp_dir)
        subprocess.run(
            [
                "7z",
                "x",
                "-y",
                f"-o{temp_root}",
                str(first_test_rar),
                *selected,
            ],
            check=True,
            stdout=subprocess.DEVNULL,
        )
        first_time = None
        for index, relative_path in enumerate(selected):
            timestamp_label = Path(relative_path).name
            timestamp = datetime.strptime(timestamp_label, "%Y.%m.%d.%H.%M.%S")
            if first_time is None:
                first_time = timestamp
            relative_hours = (timestamp - first_time).total_seconds() / 3600.0
            data = np.loadtxt(temp_root / relative_path, delimiter="\t", dtype=np.float64)
            if data.ndim != 2 or data.shape[1] < 8:
                raise SystemExit(
                    f"{dataset}: expected eight vibration channels in {relative_path}, got {data.shape}"
                )
            bearing3 = data[:, 4:6].reshape(-1)
            bearing4 = data[:, 6:8].reshape(-1)
            rows.append(
                {
                    "step": index,
                    "timestamp": timestamp.isoformat(),
                    "relative_hours": relative_hours,
                    "source_file": timestamp_label,
                    "bearing3_rms": rms(bearing3),
                    "bearing4_rms": rms(bearing4),
                    "bearing3_kurtosis": kurtosis(bearing3),
                    "bearing4_kurtosis": kurtosis(bearing4),
                }
            )

    ensure_dir(raw_summary.parent)
    with raw_summary.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)

    write_metadata(
        DatasetSourceMetadata(
            dataset=dataset,
            source_url=dataset_source_url(dataset),
            source_archive=str(archive_path.relative_to(crate_root())),
            source_sha256=dataset_source_sha256(dataset),
            raw_summary_csv=str(raw_summary.relative_to(crate_root())),
            record_count=len(rows),
            selection_note=(
                "NASA IMS bearings set 1 only. The summary uses the last 64 one-second snapshots "
                "from 1st_test, collapsed into RMS and kurtosis features for the failing bearing 3 "
                "and bearing 4 channel groups (channels 5-8 in the IMS readme)."
            ),
        )
    )
    print(f"{dataset}: wrote raw summary {raw_summary}")


def fetch_dataset(dataset: str, force_download: bool, force_regenerate: bool) -> None:
    raw_summary = dataset_raw_summary_path(dataset)
    if existing_raw_summary_is_usable(dataset) and not force_regenerate:
        archive_path = dataset_archive_path(dataset)
        if archive_path.is_file():
            actual = sha256_file(archive_path)
            expected = dataset_source_sha256(dataset)
            if actual != expected:
                raise SystemExit(
                    f"{dataset}: source archive checksum mismatch: expected {expected}, got {actual}"
                )
        print(f"{dataset}: using existing checked-in raw summary cache at {raw_summary}")
        return

    maybe_download_archive(dataset, force_download=force_download)
    if dataset == "nasa_milling":
        extract_milling_raw_summary(force_regenerate=True)
    elif dataset == "nasa_bearings":
        extract_bearings_raw_summary(force_regenerate=True)
    else:
        raise SystemExit(f"unsupported dataset {dataset}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Fetch the real NASA public dataset archives and write deterministic raw-summary slices"
    )
    parser.add_argument(
        "--dataset",
        action="append",
        choices=DATASETS,
        help="Dataset to fetch. Defaults to both NASA demo datasets.",
    )
    parser.add_argument(
        "--force-download",
        action="store_true",
        help="Re-download the source archive even if a verified local copy exists.",
    )
    parser.add_argument(
        "--force-regenerate",
        action="store_true",
        help="Regenerate the deterministic raw summary CSV even if it is already present.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    datasets = args.dataset or list(DATASETS)
    for dataset in datasets:
        fetch_dataset(
            dataset,
            force_download=args.force_download,
            force_regenerate=args.force_regenerate,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
