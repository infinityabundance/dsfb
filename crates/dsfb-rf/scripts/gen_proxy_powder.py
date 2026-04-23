#!/usr/bin/env python3
"""
POWDER synthetic-proxy slice generator.

Schema-preserving stand-in (<=2 MB) for a Genesys Lab POWDER LTE Band 7
capture. Signal reuses the crate's urban-multipath synthetic model
(see examples/urban_multipath_prognosis.rs).
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import numpy as np

SAMPLE_RATE = 7_690_000.0      # matches POWDER Globecom metadata
CENTER_HZ = 2_685_000_000      # LTE Band 7 downlink
N_SAMPLES = 262_144            # 262 144 cf32 samples = 2 MB


def generate(
    out_json: Path,
    out_bin: Path,
    rng: np.random.Generator,
    n_samples: int = N_SAMPLES,
) -> None:
    n_samples = int(n_samples)

    # Wide-band OFDM proxy (LTE-ish 20 MHz PHY downsampled to 7.69 MS/s).
    n_sub = 128
    n_sym = int(np.ceil(n_samples / n_sub))
    qam16 = (rng.choice([-3, -1, 1, 3], size=(n_sym, n_sub - 16)) +
             1j * rng.choice([-3, -1, 1, 3], size=(n_sym, n_sub - 16))) / np.sqrt(10.0)
    grid = np.zeros((n_sym, n_sub), dtype=np.complex64)
    grid[:, 8:-8] = qam16.astype(np.complex64)
    time_grid = np.fft.ifft(grid, axis=1).astype(np.complex64)
    tx = time_grid.reshape(-1)[:n_samples]

    # Urban multipath: sparse-tap Rayleigh with log-normal shadowing.
    n_taps = 6
    tap_delays = rng.integers(0, 64, size=n_taps)
    tap_gains_db = -rng.exponential(scale=6.0, size=n_taps)
    tap_gains = 10 ** (tap_gains_db / 20.0)
    tap_phases = rng.uniform(0, 2 * np.pi, size=n_taps)
    channel = np.zeros(n_samples, dtype=np.complex64)
    for d, g, ph in zip(tap_delays, tap_gains, tap_phases):
        if d < n_samples:
            channel[d:] += g * np.exp(1j * ph) * tx[: n_samples - d]

    shadow_db = rng.standard_normal(1)[0] * 4.0
    channel *= 10 ** (shadow_db / 20.0)

    noise_sigma = 10 ** (-25 / 20)
    noisy = channel + noise_sigma * (rng.standard_normal(n_samples) + 1j * rng.standard_normal(n_samples)) / np.sqrt(2)

    cf32 = np.empty(n_samples * 2, dtype=np.float32)
    cf32[0::2] = noisy.real.astype(np.float32)
    cf32[1::2] = noisy.imag.astype(np.float32)
    out_bin.write_bytes(cf32.tobytes())

    meta = {
        "dsfb_rf:provenance": "synthetic-proxy",
        "dsfb_rf:proxy_for": "POWDER Globecom 4G LTE Band 7 capture (Genesys Lab)",
        "dsfb_rf:source_model": "LTE-flavoured OFDM + 6-tap Rayleigh + log-normal shadowing; see examples/urban_multipath_prognosis.rs",
        "dsfb_rf:generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "dsfb_rf:n_samples": n_samples,
        "global": {
            "core:datatype": "cf32",
            "core:sample_rate": str(int(SAMPLE_RATE)),
            "core:version": "0.0.1",
            "core:record_date": time.strftime("%b %d, %Y", time.gmtime()),
            "core:description": (
                "[SYNTHETIC PROXY] Schema-preserving stand-in for a POWDER "
                "capture when the real 4.3 GB zip is absent."
            ),
        },
        "captures": {
            "core:sample_start": 0,
            "core:center_frequency": str(CENTER_HZ),
            "core:band": "LTE Band 7",
            "core:set": "1",
            "core:day": "1",
        },
        "annotations": {
            "core:sample_start": 0,
            "core:sample_count": str(n_samples),
            "core:environment": "[SYNTHETIC PROXY] urban multipath model",
            "core:protocol": "4G",
            "transmitter": {"core:location": "proxy_bes", "core:radio": "Ettus USRP X310"},
            "receiver": {"core:location": "proxy_humanities", "core:radio": "Ettus USRP B210"},
        },
    }
    out_json.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
