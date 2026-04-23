#!/usr/bin/env python3
"""
DeepBeam slice generator.

Primary path: ``extract_local_slice`` reads a user-downloaded DeepBeam
HDF5 file (e.g. ``neu_ww72bk394.h5`` from the Northeastern repository
collection ``neu:ww72bh952``) and emits a ≤ 2 MB HDF5 head slice with
the parent's native schema: ``/iq[time,2]``, ``/gain[time]``,
``/rx_beam[time]``, ``/tx_beam[time]``.

Fallback path: ``generate`` — schema-preserving 60 GHz mmWave proxy
(Golay-training + Rician LoS + scatterer + beam-squint) emitted only
when the local file is absent, loudly stamped ``[SYNTHETIC PROXY]``.
"""

from __future__ import annotations

import hashlib
import json
import time
from pathlib import Path

import numpy as np

DEEPBEAM_HEAD_N = 8192  # 8192 iq pairs ≈ 131 KB float64 + small telemetry → ≪ 2 MB

SAMPLE_RATE = 3_840_000_000 / 64   # ~60 MS/s complex baseband (mmWave PHY / 64)
CENTER_HZ = 60_480_000_000
N_SAMPLES = 131_072  # ~1 MB cf32


def generate(
    out_bin: Path,
    out_meta: Path,
    rng: np.random.Generator,
    n_samples: int = N_SAMPLES,
) -> None:
    n_samples = int(n_samples)
    tx_ant = int(rng.integers(0, 4))
    rx_ant = int(rng.integers(0, 4))

    # 802.11ad-ish Golay complementary pair as training field.
    ga = np.array([1, 1, 1, -1, 1, 1, -1, 1, 1, 1, 1, -1, -1, -1, 1, -1], dtype=np.complex64)
    gb = np.array([1, 1, 1, -1, 1, 1, -1, 1, -1, -1, -1, 1, 1, 1, -1, 1], dtype=np.complex64)
    training = np.concatenate([ga, gb] * 32)
    data = rng.choice([-1, 1, -1j, 1j], size=n_samples).astype(np.complex64)
    data[: training.size] = training[: min(training.size, n_samples)]
    tx = data[:n_samples]

    # Rician LoS with K=10 dB plus one strong scattered tap.
    k_lin = 10 ** (10 / 10)
    los_phase = rng.uniform(0, 2 * np.pi)
    los = np.sqrt(k_lin / (k_lin + 1)) * np.exp(1j * los_phase)
    scat_phase = rng.uniform(0, 2 * np.pi)
    scat_gain = 0.3
    rx = los * tx
    scat_delay = int(rng.integers(4, 32))
    if scat_delay < n_samples:
        rx[scat_delay:] += scat_gain * np.exp(1j * scat_phase) * tx[: n_samples - scat_delay]

    # Beam-squint: small frequency-dependent phase ramp.
    squint_ramp = np.exp(1j * 2 * np.pi * 1e-5 * np.arange(n_samples)).astype(np.complex64)
    rx = rx * squint_ramp

    noise_sigma = 10 ** (-22 / 20)
    noise_re = rng.standard_normal(n_samples).astype(np.float32) / np.sqrt(2)
    noise_im = rng.standard_normal(n_samples).astype(np.float32) / np.sqrt(2)
    rx = rx + noise_sigma * (noise_re + 1j * noise_im).astype(np.complex64)

    cf32 = np.empty(n_samples * 2, dtype=np.float32)
    cf32[0::2] = rx.real.astype(np.float32)
    cf32[1::2] = rx.imag.astype(np.float32)
    out_bin.write_bytes(cf32.tobytes())

    meta = {
        "dsfb_rf:provenance": "synthetic-proxy",
        "dsfb_rf:proxy_for": "DeepBeam 60 GHz mmWave dataset (Northeastern neu:ww72bh952)",
        "dsfb_rf:source_model": "802.11ad-flavoured Golay training + Rician LoS (K=10 dB) + single scatterer + beam-squint ramp",
        "dsfb_rf:generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "sample_rate_hz": SAMPLE_RATE,
        "center_frequency_hz": CENTER_HZ,
        "datatype": "cf32 (little-endian interleaved float32 I/Q)",
        "n_samples": n_samples,
        "tx_antenna": tx_ant,
        "rx_antenna": rx_ant,
        "notice": "[SYNTHETIC PROXY] DeepBeam stand-in; not an NI transceiver capture.",
    }
    out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")


def extract_local_slice(
    src_h5: Path,
    out_h5: Path,
    out_meta: Path,
    n_head: int = DEEPBEAM_HEAD_N,
) -> dict:
    """
    Extract a ≤ 2 MB head slice from a user-downloaded DeepBeam HDF5 file.

    Parent schema (established via earlier HTTP Range probe of
    ``neu_ww72bk394.h5``):
        /iq        float64  shape = (~11B, 2)    gzip chunked
        /gain      float64  shape = (~11B,)      gzip chunked
        /rx_beam   float64  shape = (~11B,)      gzip chunked
        /tx_beam   float64  shape = (~11B,)      gzip chunked

    Emits the first ``n_head`` rows of each preserving dtype, yielding a
    tens-of-KB head slice with native NI-transceiver provenance.
    """
    try:
        import h5py  # type: ignore
    except ImportError as e:
        raise RuntimeError(
            "h5py is required for DeepBeam local slice extraction; "
            "install via `pip install h5py`"
        ) from e

    src_h5 = Path(src_h5)
    if not src_h5.is_file():
        raise FileNotFoundError(f"DeepBeam HDF5 not found: {src_h5}")

    out_h5 = Path(out_h5)
    out_meta = Path(out_meta)

    with h5py.File(src_h5, "r") as src:
        expected = {"iq", "gain", "rx_beam", "tx_beam"}
        have = set(src.keys())
        missing = expected - have
        if missing:
            raise RuntimeError(
                f"DeepBeam parent file missing expected datasets: {missing} "
                f"(have: {sorted(have)})"
            )
        iq_total = src["iq"].shape[0]
        n = min(int(n_head), iq_total)
        iq = src["iq"][:n]
        gain = src["gain"][:n]
        rx_beam = src["rx_beam"][:n]
        tx_beam = src["tx_beam"][:n]

    with h5py.File(out_h5, "w") as f:
        f.create_dataset("iq", data=iq, compression="gzip", compression_opts=4)
        f.create_dataset("gain", data=gain, compression="gzip", compression_opts=4)
        f.create_dataset("rx_beam", data=rx_beam, compression="gzip", compression_opts=4)
        f.create_dataset("tx_beam", data=tx_beam, compression="gzip", compression_opts=4)
        f.attrs["dsfb_rf:provenance"] = "real-local-file"
        f.attrs["dsfb_rf:source"] = (
            "DeepBeam Northeastern repository collection neu:ww72bh952 "
            "(user-downloaded HDF5 head slice)"
        )
        f.attrs["dsfb_rf:parent_file"] = src_h5.name
        # Fast partial digest to pin parent identity without hashing ~50 GB
        h = hashlib.sha256()
        with src_h5.open("rb") as g:
            h.update(g.read(4 * 1024 * 1024))  # first 4 MiB
        f.attrs["dsfb_rf:parent_first4MiB_sha256"] = h.hexdigest()
        f.attrs["dsfb_rf:parent_iq_rows_total"] = iq_total
        f.attrs["dsfb_rf:head_n_samples"] = n
        f.attrs["dsfb_rf:extracted_at"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        f.attrs["dsfb_rf:schema"] = (
            "iq float64 (N,2); gain float64 (N,); rx_beam float64 (N,); tx_beam float64 (N,); "
            "NI mmWave transceiver native layout"
        )

    meta = {
        "dsfb_rf:provenance": "real-local-file",
        "dsfb_rf:source": "DeepBeam (Northeastern neu:ww72bh952) — user-downloaded HDF5",
        "dsfb_rf:parent_file": src_h5.name,
        "dsfb_rf:parent_iq_rows_total": int(iq_total),
        "dsfb_rf:head_n_samples": int(n),
        "dsfb_rf:schema": {
            "iq": "float64 (N,2) — NI transceiver I/Q pairs",
            "gain": "float64 (N,) — per-sample RF gain",
            "rx_beam": "float64 (N,) — receive beam index",
            "tx_beam": "float64 (N,) — transmit beam index",
        },
        "dsfb_rf:extracted_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "notice": (
            "Real DeepBeam 60 GHz mmWave head slice; not a benchmark reproduction. "
            "Contextual residual-trace exhibit only."
        ),
    }
    out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    return meta
