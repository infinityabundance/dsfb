#!/usr/bin/env python3
"""
DeepSense-6G slice generator.

Primary path: ``extract_scenario23_slice`` reads a user-downloaded
Scenario-23 UAV-mmWave zip (``scenario23_dev_w_resources.zip`` from
deepsense6g.net) and emits a ≤ 2 MB HDF5 head slice with
``[time, beam]`` mmWave power + UAV telemetry.

Fallback path: ``generate`` — schema-preserving multimodal proxy (mmWave IQ
block + GPS trace + camera hash surrogate) emitted only when the local zip
is absent, loudly stamped ``[SYNTHETIC PROXY]``.
"""

from __future__ import annotations

import hashlib
import io
import json
import time
import zipfile
from pathlib import Path

import numpy as np

N_MMWAVE_SAMPLES = 65_536   # 0.5 MB cf32
N_GPS_SAMPLES = 1200
SCENARIO23_HEAD_N = 1000    # first 1000 UAV samples: ~0.26 MB mmWave + ~20 KB telemetry


def generate(
    out_bin: Path,
    out_meta: Path,
    rng: np.random.Generator,
    n_mmwave: int = N_MMWAVE_SAMPLES,
    n_gps: int = N_GPS_SAMPLES,
) -> None:
    n_mmwave = int(n_mmwave)
    n_gps = int(n_gps)

    # mmWave IQ block: Rayleigh + slow fading.
    t = np.arange(n_mmwave)
    h = (rng.standard_normal(n_mmwave) + 1j * rng.standard_normal(n_mmwave)).astype(np.complex64) / np.sqrt(2.0)
    slow = 0.5 + 0.5 * np.cos(2 * np.pi * t * 1e-4 + rng.uniform(0, 2 * np.pi))
    h = h * slow.astype(np.float32)
    cf32 = np.empty(n_mmwave * 2, dtype=np.float32)
    cf32[0::2] = h.real.astype(np.float32)
    cf32[1::2] = h.imag.astype(np.float32)

    # Fake deterministic camera hash (we never fabricate pixels).
    cam_hash = hashlib.sha256(cf32.tobytes()[:4096]).hexdigest()

    # NMEA-like GPS trace — linear walk around a fixed lat/lon.
    lat0, lon0 = 42.3399, -71.0869  # Northeastern-ish
    lat = lat0 + (np.arange(n_gps) * 1e-6 + rng.standard_normal(n_gps).cumsum() * 1e-7)
    lon = lon0 + (np.arange(n_gps) * 1e-6 + rng.standard_normal(n_gps).cumsum() * 1e-7)
    gps_rows = [f"{i*0.1:.1f},{lat[i]:.6f},{lon[i]:.6f}" for i in range(n_gps)]

    payload = cf32.tobytes() + b"\n---GPS---\n" + "\n".join(gps_rows).encode("utf-8")
    out_bin.write_bytes(payload)

    meta = {
        "dsfb_rf:provenance": "synthetic-proxy",
        "dsfb_rf:proxy_for": "DeepSense-6G scenario sample (deepsense6g.net)",
        "dsfb_rf:source_model": "Rayleigh mmWave fading + slow amplitude modulation + linear GPS walk + sha256 camera hash surrogate",
        "dsfb_rf:generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "modalities": {
            "mmwave_iq": {"datatype": "cf32", "n_samples": n_mmwave},
            "gps_trace": {"n_samples": n_gps, "format": "CSV: t_s,lat_deg,lon_deg"},
            "camera_hash": cam_hash,
        },
        "notice": "[SYNTHETIC PROXY] No real imagery fabricated; camera modality represented by a deterministic hash surrogate.",
    }
    out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")


def _read_scalar_txt(zf: zipfile.ZipFile, path: str) -> float:
    try:
        return float(zf.read(path).decode("ascii").strip().splitlines()[0])
    except (KeyError, ValueError):
        return float("nan")


def extract_scenario23_slice(
    zip_path: Path,
    out_h5: Path,
    out_meta: Path,
    n_head: int = SCENARIO23_HEAD_N,
) -> dict:
    """
    Extract a ≤ 2 MB head slice from a user-downloaded Scenario 23 zip.

    Schema of the emitted HDF5 file:
        /mmwave_power[time, beam]  float32   shape = (n_head, 64)
        /best_beam_index[time]     int16     shape = (n_head,)
        /altitude[time]            float32   shape = (n_head,)
        /speed[time]               float32   shape = (n_head,)
        /pitch[time]               float32   shape = (n_head,)
        /roll[time]                float32   shape = (n_head,)
        /distance[time]            float32   shape = (n_head,)
        /height[time]              float32   shape = (n_head,)

    Provenance stamped as HDF5 root attributes so a reviewer can inspect via
    ``h5dump -A``. Returns a small manifest dict for SLICE_MANIFEST.json.
    """
    import csv  # stdlib; optional dependency isolation

    try:
        import h5py  # type: ignore
    except ImportError as e:
        raise RuntimeError(
            "h5py is required for DeepSense-6G Scenario 23 real slice extraction; "
            "install via `pip install h5py`"
        ) from e

    zip_path = Path(zip_path)
    if not zip_path.is_file():
        raise FileNotFoundError(f"Scenario 23 zip not found: {zip_path}")

    out_h5 = Path(out_h5)
    out_meta = Path(out_meta)

    with zipfile.ZipFile(zip_path, "r") as zf:
        csv_bytes = zf.read("scenario23_dev/scenario23.csv")
        reader = csv.DictReader(io.StringIO(csv_bytes.decode("utf-8")))
        rows = []
        for i, row in enumerate(reader):
            if i >= n_head:
                break
            rows.append(row)
        if not rows:
            raise RuntimeError("scenario23.csv is empty")

        n = len(rows)
        mmwave = np.zeros((n, 64), dtype=np.float32)
        best_beam = np.zeros(n, dtype=np.int16)
        altitude = np.zeros(n, dtype=np.float32)
        speed = np.zeros(n, dtype=np.float32)
        pitch = np.zeros(n, dtype=np.float32)
        roll = np.zeros(n, dtype=np.float32)
        distance = np.zeros(n, dtype=np.float32)
        height = np.zeros(n, dtype=np.float32)

        for k, row in enumerate(rows):
            mm_path = "scenario23_dev/" + row["unit1_pwr_60ghz"].lstrip("./")
            try:
                mm = zf.read(mm_path).decode("ascii").strip().splitlines()
                vec = np.array([float(x) for x in mm], dtype=np.float32)
                if vec.size != 64:
                    raise ValueError(f"expected 64 beams, got {vec.size} in {mm_path}")
                mmwave[k] = vec
            except (KeyError, ValueError) as err:
                raise RuntimeError(f"mmWave read failed at row {k}: {err}") from err

            try:
                best_beam[k] = int(row["unit1_beam_index"])
            except (KeyError, ValueError):
                best_beam[k] = -1

            altitude[k] = _read_scalar_txt(zf, "scenario23_dev/" + row["unit2_altitude"].lstrip("./"))
            speed[k] = _read_scalar_txt(zf, "scenario23_dev/" + row["unit2_speed"].lstrip("./"))
            pitch[k] = _read_scalar_txt(zf, "scenario23_dev/" + row["unit2_pitch"].lstrip("./"))
            roll[k] = _read_scalar_txt(zf, "scenario23_dev/" + row["unit2_roll"].lstrip("./"))
            distance[k] = _read_scalar_txt(zf, "scenario23_dev/" + row["unit2_distance"].lstrip("./"))
            height[k] = _read_scalar_txt(zf, "scenario23_dev/" + row["unit2_height"].lstrip("./"))

    with h5py.File(out_h5, "w") as f:
        f.create_dataset("mmwave_power", data=mmwave, compression="gzip", compression_opts=4)
        f.create_dataset("best_beam_index", data=best_beam, compression="gzip", compression_opts=4)
        f.create_dataset("altitude", data=altitude, compression="gzip", compression_opts=4)
        f.create_dataset("speed", data=speed, compression="gzip", compression_opts=4)
        f.create_dataset("pitch", data=pitch, compression="gzip", compression_opts=4)
        f.create_dataset("roll", data=roll, compression="gzip", compression_opts=4)
        f.create_dataset("distance", data=distance, compression="gzip", compression_opts=4)
        f.create_dataset("height", data=height, compression="gzip", compression_opts=4)
        f.attrs["dsfb_rf:provenance"] = "real-local-zip"
        f.attrs["dsfb_rf:source"] = (
            "DeepSense-6G Scenario 23 UAV mmWave "
            "(deepsense6g.net/scenarios/scenario-23) — user-downloaded head slice"
        )
        f.attrs["dsfb_rf:parent_zip"] = zip_path.name
        f.attrs["dsfb_rf:parent_zip_sha256"] = hashlib.sha256(zip_path.read_bytes()[:64]).hexdigest() + "-first64B"
        f.attrs["dsfb_rf:head_n_samples"] = n
        f.attrs["dsfb_rf:extracted_at"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        f.attrs["dsfb_rf:schema"] = (
            "mmwave_power[time,beam] float32 (N,64); best_beam_index int16 (N,); "
            "telemetry (altitude, speed, pitch, roll, distance, height) float32 (N,)"
        )

    meta = {
        "dsfb_rf:provenance": "real-local-zip",
        "dsfb_rf:source": "DeepSense-6G Scenario 23 UAV mmWave (deepsense6g.net)",
        "dsfb_rf:parent_zip": zip_path.name,
        "dsfb_rf:head_n_samples": n,
        "dsfb_rf:schema": {
            "mmwave_power": "float32 (N,64) — 60 GHz beamformed power per beam",
            "best_beam_index": "int16 (N,) — ground-truth best-beam label",
            "altitude": "float32 (N,) — UAV altitude (meters)",
            "speed": "float32 (N,) — UAV speed (m/s)",
            "pitch": "float32 (N,) — UAV pitch (rad)",
            "roll": "float32 (N,) — UAV roll (rad)",
            "distance": "float32 (N,) — UAV-BS distance (m)",
            "height": "float32 (N,) — UAV height AGL (m)",
        },
        "dsfb_rf:extracted_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "notice": (
            "Real DeepSense-6G Scenario 23 (UAV mmWave) head slice; "
            "not a benchmark reproduction. Contextual residual-trace exhibit only."
        ),
    }
    out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    return meta
