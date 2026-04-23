#!/usr/bin/env python3
"""
ORACLE synthetic-proxy slice generator.

Emits a schema-preserving <=2 MB SigMF pair that stands in for the real
Northeastern GENESYS ORACLE USRP X310 capture when the 28 GB zip is not
present on the local filesystem and the public mirror is gated.

Honesty contract: the emitted .sigmf-meta always carries
`dsfb_rf:provenance="synthetic-proxy"`, and the slice is never substituted
for a paper metric. Signal content is drawn from the crate's existing
WiFi-with-device-fingerprint impairment model (see examples/oracle_usrp_b200.rs).
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import numpy as np

SAMPLE_RATE = 5_000_000.0   # 5 MS/s (matches ORACLE)
CENTER_HZ = 2_450_000_000   # 2.45 GHz WiFi (matches ORACLE)
N_SAMPLES = 131_072         # 131 072 complex64 = 1 MB


def generate(
    out_meta: Path,
    out_data: Path,
    rng: np.random.Generator,
    n_samples: int = N_SAMPLES,
) -> None:
    n_samples = int(n_samples)

    # OFDM-like in-band signal (64 subcarriers, 52 occupied — WiFi-ish) +
    # emitter fingerprint impairments: LO leakage, I/Q amplitude imbalance,
    # I/Q phase skew, DC offset, Tx nonlinearity (soft compression).
    n_subcarriers = 64
    n_occupied = 52
    symbols = int(np.ceil(n_samples / n_subcarriers))
    qam4 = rng.choice([-1 - 1j, -1 + 1j, 1 - 1j, 1 + 1j], size=(symbols, n_occupied))
    freq_grid = np.zeros((symbols, n_subcarriers), dtype=np.complex64)
    occ_idx = np.r_[6:32, 33:59]  # exclude DC and edges
    freq_grid[:, occ_idx] = qam4.astype(np.complex64)
    time_grid = np.fft.ifft(freq_grid, axis=1).astype(np.complex64)
    signal = time_grid.reshape(-1)[:n_samples]

    # Device fingerprint (per-emitter): unique per rng seed.
    lo_leak = 0.003 + 0.002 * rng.random()
    amp_imbal_db = 0.05 + 0.05 * rng.random()
    phase_skew_deg = 0.5 + 0.5 * rng.random()
    dc_i = 0.002 * (rng.random() - 0.5)
    dc_q = 0.002 * (rng.random() - 0.5)

    amp_imbal = 10 ** (amp_imbal_db / 20.0)
    phase_skew = np.deg2rad(phase_skew_deg)
    signal = signal.real * amp_imbal + 1j * (signal.imag * np.cos(phase_skew) + signal.real * np.sin(phase_skew))
    signal = signal + (dc_i + 1j * dc_q) + lo_leak * np.exp(1j * 2 * np.pi * 0.01 * np.arange(n_samples))

    # Soft Tx compression.
    mag = np.abs(signal)
    sat = mag / np.sqrt(1.0 + (mag / 0.8) ** 2)
    signal = (signal / np.maximum(mag, 1e-12)) * sat

    # AWGN at ~30 dB SNR.
    noise_sigma = 10 ** (-30 / 20)
    signal = signal + noise_sigma * (rng.standard_normal(n_samples) + 1j * rng.standard_normal(n_samples)) / np.sqrt(2)

    cf32 = np.empty(n_samples * 2, dtype=np.float32)
    cf32[0::2] = signal.real.astype(np.float32)
    cf32[1::2] = signal.imag.astype(np.float32)
    out_data.write_bytes(cf32.tobytes())

    meta = {
        "dsfb_rf:provenance": "synthetic-proxy",
        "dsfb_rf:proxy_for": "ORACLE Northeastern GENESYS USRP X310 WiFi Raw IQ",
        "dsfb_rf:source_model": "WiFi-flavoured OFDM + USRP fingerprint impairment (LO leakage, I/Q imbalance, phase skew, DC offset, Tx soft compression); see examples/oracle_usrp_b200.rs",
        "dsfb_rf:generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "dsfb_rf:n_samples": n_samples,
        "global": {
            "core:datatype": "cf32",
            "core:sample_rate": float(SAMPLE_RATE),
            "core:version": "0.02",
            "core:author": "dsfb-rf synthetic proxy",
            "core:description": (
                "[SYNTHETIC PROXY] Schema-preserving stand-in for an ORACLE "
                "SigMF capture when the real 28 GB zip is absent. Do not cite "
                "as a real measurement."
            ),
        },
        "captures": [{"core:sample_start": 0, "frequency": CENTER_HZ, "core:time": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}],
        "annotations": [
            {
                "core:sample_start": 0,
                "core:sample_count": n_samples,
                "genesys:transmitter": {
                    "model": "[SYNTHETIC PROXY] Ettus USRP X310 with UBX-160",
                    "device_id": f"PROXY-{int(rng.integers(0xFFFFFF)):06X}",
                },
                "genesys:receiver": {"model": "[SYNTHETIC PROXY] Ettus USRP B210"},
            }
        ],
    }
    out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
