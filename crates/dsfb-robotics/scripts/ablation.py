#!/usr/bin/env python3
"""Ablation study for the DSFB FSM components.

Three ablations, each disabling one structural element of the
parametric Python FSM (which has been validated against the Rust
canonical engine):

  A1 — drift detection disabled (treat ṙ as zero):
       drops the SustainedOutwardDrift Boundary trigger; remaining
       Boundary episodes come from AbruptSlewViolation or
       RecurrentBoundaryGrazing.

  A2 — slew detection disabled (treat r̈ as zero):
       drops the AbruptSlewViolation Boundary trigger; remaining
       Boundary episodes come from outward drift or grazing.

  A3 — hysteresis disabled (commit on first sample):
       removes 2-confirmation hysteresis; the FSM commits the raw
       state immediately. Quantifies how many spurious-state samples
       hysteresis was suppressing.

For each ablation the grammar census on the chosen dataset is computed
and compared against the canonical-FSM census from the same dataset.
The output JSON reports the absolute and relative shift in
{Admissible, Boundary, Violation, compression_ratio} per ablation.

Default targets: panda_gaz (kinematics), cwru (PHM), icub_pushrecovery
(balancing) — one exemplar per residual family. Pass slugs as args to
override.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

CRATE_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(CRATE_ROOT / "scripts"))

from dsfb_fsm_parametric import FsmParams, load_residual_stream, run_fsm  # noqa: E402

PROCESSED_ROOT = CRATE_ROOT / "data" / "processed"
OUT_ROOT = CRATE_ROOT / "audit" / "ablation"

DEFAULT_SLUGS = ["panda_gaz", "cwru", "icub_pushrecovery"]


def ablation_for(slug: str) -> dict:
    pub = PROCESSED_ROOT / f"{slug}_published.csv"
    base = PROCESSED_ROOT / f"{slug}.csv"
    target = pub if pub.is_file() else base
    if not target.is_file():
        raise FileNotFoundError(f"{slug}: residual CSV missing")
    stream = load_residual_stream(str(target))

    canonical = run_fsm(stream, FsmParams())
    a1_no_drift = run_fsm(stream, FsmParams(disable_drift=True))
    a2_no_slew = run_fsm(stream, FsmParams(disable_slew=True))
    a3_no_hyst = run_fsm(stream, FsmParams(disable_hysteresis=True))

    def shift(ablated: dict) -> dict:
        return {
            "admissible_delta": ablated["admissible"] - canonical["admissible"],
            "boundary_delta": ablated["boundary"] - canonical["boundary"],
            "violation_delta": ablated["violation"] - canonical["violation"],
            "compression_delta": ablated["compression_ratio"] - canonical["compression_ratio"],
        }

    return {
        "dataset": slug,
        "residual_source": "published-theta" if pub.is_file() else "early-window-nominal",
        "canonical": canonical,
        "ablations": {
            "A1_drift_disabled": {
                "census": a1_no_drift,
                "shift_vs_canonical": shift(a1_no_drift),
                "interpretation": (
                    "When drift is silenced, the SustainedOutwardDrift Boundary trigger is "
                    "removed. The boundary_delta below quantifies how many Boundary episodes "
                    "the canonical FSM credited specifically to outward drift."
                ),
            },
            "A2_slew_disabled": {
                "census": a2_no_slew,
                "shift_vs_canonical": shift(a2_no_slew),
                "interpretation": (
                    "When slew is silenced, the AbruptSlewViolation Boundary trigger is "
                    "removed. The boundary_delta below quantifies the Boundary contribution "
                    "of curvature-driven (abrupt onset) episodes."
                ),
            },
            "A3_hysteresis_disabled": {
                "census": a3_no_hyst,
                "shift_vs_canonical": shift(a3_no_hyst),
                "interpretation": (
                    "When hysteresis is removed, the FSM commits the raw state immediately. "
                    "The deltas below quantify the volume of single-sample state flips that "
                    "the canonical 2-confirmation hysteresis was suppressing."
                ),
            },
        },
    }


def main() -> int:
    targets = sys.argv[1:] or DEFAULT_SLUGS
    OUT_ROOT.mkdir(parents=True, exist_ok=True)
    failures = []
    for slug in targets:
        print(f"== {slug} ==", flush=True)
        try:
            data = ablation_for(slug)
        except Exception as exc:
            print(f"FAIL {slug}: {exc}")
            failures.append(slug)
            continue
        out_path = OUT_ROOT / f"{slug}_ablation.json"
        with out_path.open("w") as fh:
            json.dump(data, fh, indent=2)
            fh.write("\n")
        c = data["canonical"]
        a1 = data["ablations"]["A1_drift_disabled"]["shift_vs_canonical"]
        a2 = data["ablations"]["A2_slew_disabled"]["shift_vs_canonical"]
        a3 = data["ablations"]["A3_hysteresis_disabled"]["shift_vs_canonical"]
        print(f"  canonical:  A={c['admissible']}  B={c['boundary']}  V={c['violation']}  comp={c['compression_ratio']:.3f}")
        print(f"  no-drift:   ΔB={a1['boundary_delta']:+}  Δcomp={a1['compression_delta']:+.3f}")
        print(f"  no-slew:    ΔB={a2['boundary_delta']:+}  Δcomp={a2['compression_delta']:+.3f}")
        print(f"  no-hyst:    ΔB={a3['boundary_delta']:+}  Δcomp={a3['compression_delta']:+.3f}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
