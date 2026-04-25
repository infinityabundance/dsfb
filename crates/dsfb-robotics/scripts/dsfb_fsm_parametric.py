#!/usr/bin/env python3
"""Parametric DSFB FSM replica.

Mirrors the Rust `dsfb_robotics::engine::DsfbRoboticsEngine<W, K>` and
`figures_real.run_dsfb`, but with W, K, beta, delta_s, and per-component
disable flags exposed as runtime parameters. Used by `sensitivity_grid.py`
and `ablation.py`. Validated against the Rust binary for the canonical
parameter set (W=8, K=4, beta=0.5, delta_s=0.05).
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np


@dataclass
class FsmParams:
    W: int = 8
    K: int = 4
    boundary_frac: float = 0.5
    delta_s: float = 0.05
    disable_drift: bool = False        # T6.3 ablation A1
    disable_slew: bool = False         # T6.3 ablation A2
    disable_hysteresis: bool = False   # T6.3 ablation A3


def _check_grazing(hits: list[bool], hit_count: int, K: int) -> str:
    if hit_count >= K and sum(hits) >= K:
        return "Boundary"
    return "Admissible"


def _calibrate_envelope(residuals: np.ndarray) -> float:
    n = len(residuals)
    if n == 0:
        return float("inf")
    cal_n = max(1, n // 5)
    cal = residuals[:cal_n]
    finite = cal[np.isfinite(cal)]
    if finite.size == 0:
        return float("inf")
    mu = float(np.mean(np.abs(finite)))
    sigma = float(np.std(np.abs(finite)))
    return mu + 3.0 * sigma


def run_fsm(residuals: np.ndarray, params: FsmParams) -> dict:
    """Run the parametric FSM, return aggregate census.

    Drift/slew accumulation follows the Rust `sign::SignWindow::push`
    summation order exactly so the Python census matches the Rust binary
    bit-for-bit on canonical-parameter runs.
    """
    rho = _calibrate_envelope(residuals)
    W = params.W
    # Ring buffer mirroring the Rust SignWindow.
    norms = [0.0] * W
    head = 0
    count = 0
    prev_drift = 0.0
    pending = "Admissible"
    confirms = 0
    committed = "Admissible"
    boundary_hits = [False] * params.K
    hit_head = 0
    hit_count = 0

    admissible = boundary = violation = 0
    max_norm_sq = 0.0

    for r in residuals:
        below_floor = not np.isfinite(r)
        norm = abs(r) if np.isfinite(r) else 0.0
        norm_sq = norm * norm
        if norm_sq > max_norm_sq:
            max_norm_sq = norm_sq

        # Insert into the ring buffer using Rust SignWindow ordering.
        norms[head] = norm
        head = (head + 1) % W
        if count < W:
            count += 1

        if below_floor or count < 2:
            drift = 0.0
            slew = 0.0
            prev_drift = 0.0
        else:
            # Mean first-difference accumulated in the same order Rust
            # does it: walk backwards from the most-recent sample.
            filled = min(count, W)
            sum_diff = 0.0
            n_diffs = 0
            i = 1
            while i < filled:
                cur = (head + W - 1 - (i - 1)) % W
                prev = (head + W - 1 - i) % W
                sum_diff += norms[cur] - norms[prev]
                n_diffs += 1
                i += 1
            drift = sum_diff / n_diffs if n_diffs > 0 else 0.0
            slew = drift - prev_drift
            prev_drift = drift

        # Ablation knobs: silence drift / slew contributions.
        if params.disable_drift:
            drift = 0.0
        if params.disable_slew:
            slew = 0.0

        # Raw state.
        if norm > rho:
            raw = "Violation"
        elif norm > params.boundary_frac * rho:
            if drift > 0:
                raw = "Boundary"
            elif abs(slew) > params.delta_s:
                raw = "Boundary"
            else:
                raw = _check_grazing(boundary_hits, hit_count, params.K)
        else:
            raw = _check_grazing(boundary_hits, hit_count, params.K)

        is_approach = (norm > params.boundary_frac * rho and norm <= rho)
        boundary_hits[hit_head] = is_approach
        hit_head = (hit_head + 1) % params.K
        if hit_count < params.K:
            hit_count += 1

        # Hysteresis: 2 confirmations to commit a state change.
        if params.disable_hysteresis:
            committed = raw
        else:
            if raw == pending:
                if confirms < 2:
                    confirms += 1
                if confirms >= 2:
                    committed = raw
            else:
                pending = raw
                confirms = 1

        if committed == "Admissible":
            admissible += 1
        elif committed == "Boundary":
            boundary += 1
        elif committed == "Violation":
            violation += 1

    n = len(residuals)
    reviewed = boundary + violation
    compression = (reviewed / n) if n > 0 else 0.0
    return {
        "total_samples": n,
        "admissible": admissible,
        "boundary": boundary,
        "violation": violation,
        "compression_ratio": compression,
        "max_residual_norm_sq": max_norm_sq,
    }


def load_residual_stream(path: str) -> np.ndarray:
    rows: list[float] = []
    with open(path) as fh:
        header = fh.readline().strip()
        try:
            rows.append(float(header.split(",")[-1]))
        except ValueError:
            pass
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(float(line.split(",")[-1]))
            except ValueError:
                continue
    return np.asarray(rows, dtype=np.float64)
