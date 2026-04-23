#!/usr/bin/env python3
"""
Tampere Uni GNSS synthetic-proxy slice generator.

Schema-preserving stand-in (<=2 MB) for the Wang/Sankari/Lohan/Valkama
raw-IQ GNSS RFF dataset (Zenodo 10.5281/zenodo.13846381, CC BY 4.0) when
the 6.4 GB Data.zip is not directly reachable at notebook runtime.

Signal reuses the crate's GPS L1 C/A spoofing model (see
examples/gps_spoofing_detection.rs): clean L1 C/A baseband with optional
spoofed overlay + ionospheric scintillation impairment.
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import numpy as np

SAMPLE_RATE = 1_023_000.0 * 2     # 2x chipping rate
CENTER_HZ = 1_575_420_000         # GPS L1
N_SAMPLES = 131_072               # ~1 MB at cf32


def _ca_code(prn: int, length: int = 1023) -> np.ndarray:
    """Binary-valued (+1,-1) Gold-code-ish sequence, deterministic per prn."""
    rng = np.random.default_rng(prn * 7919 + 131)
    return np.where(rng.integers(0, 2, size=length) == 0, 1.0, -1.0).astype(np.float32)


def generate(
    out_bin: Path,
    out_meta: Path,
    rng: np.random.Generator,
    n_samples: int = N_SAMPLES,
) -> None:
    n_samples = int(n_samples)
    # Clean L1 segment, PRN 5, two samples per chip.
    code = _ca_code(prn=5, length=1023)
    upsampled = np.repeat(code, 2)
    full = np.tile(upsampled, int(np.ceil(n_samples / upsampled.size)))[:n_samples]
    doppler_hz = 2500.0
    t = np.arange(n_samples) / SAMPLE_RATE
    baseband = full.astype(np.complex64) * np.exp(1j * 2 * np.pi * doppler_hz * t).astype(np.complex64)

    # Optional spoof overlay on the second half.
    spoof_code = _ca_code(prn=12, length=1023)
    spoof_up = np.repeat(spoof_code, 2)
    spoof_full = np.tile(spoof_up, int(np.ceil(n_samples / spoof_up.size)))[:n_samples]
    spoof_t = np.arange(n_samples) / SAMPLE_RATE
    spoof_signal = spoof_full.astype(np.complex64) * np.exp(1j * 2 * np.pi * (doppler_hz + 150.0) * spoof_t).astype(np.complex64)
    spoof_mask = np.zeros(n_samples, dtype=np.float32)
    spoof_mask[n_samples // 2 :] = 1.0
    mixed = baseband + 0.6 * spoof_mask * spoof_signal

    scint_phase = rng.standard_normal(n_samples).cumsum() * 1e-3
    mixed = mixed * np.exp(1j * scint_phase).astype(np.complex64)

    noise_sigma = 10 ** (-20 / 20)
    mixed = mixed + noise_sigma * (rng.standard_normal(n_samples) + 1j * rng.standard_normal(n_samples)).astype(np.float32) / np.sqrt(2)

    cf32 = np.empty(n_samples * 2, dtype=np.float32)
    cf32[0::2] = mixed.real.astype(np.float32)
    cf32[1::2] = mixed.imag.astype(np.float32)
    out_bin.write_bytes(cf32.tobytes())

    meta = {
        "dsfb_rf:provenance": "synthetic-proxy",
        "dsfb_rf:proxy_for": "Tampere Uni GNSS RFF (Wang/Sankari/Lohan/Valkama, Zenodo 10.5281/zenodo.13846381, CC BY 4.0)",
        "dsfb_rf:source_model": "L1 C/A clean + second-half spoof overlay + ionospheric scintillation; see examples/gps_spoofing_detection.rs",
        "dsfb_rf:generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "sample_rate_hz": SAMPLE_RATE,
        "center_frequency_hz": CENTER_HZ,
        "datatype": "cf32 (little-endian interleaved float32 I/Q)",
        "n_samples": n_samples,
        "notice": "[SYNTHETIC PROXY] Head-slice stand-in for Tampere GNSS Data.zip.",
    }
    out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
