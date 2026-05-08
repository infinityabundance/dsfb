#!/usr/bin/env python3
"""DSFB-Debug figure pipeline — entry point.

Reads JSON output from the `dsfb-debug-demo` Rust binary's `results/`
directory and renders publication-quality figures into a parallel
`figures-py/` directory under the same run-id folder.

Usage:
    python3 -m tools.figures.render /path/to/output-dsfb-debug/dsfb-debug-{datetime}

Or, from inside the crate:
    cargo run --release --features demo --bin dsfb-debug-demo
    python3 -m tools.figures.render output-dsfb-debug/dsfb-debug-2026-05-07-...
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from . import _style as S
from . import architecture as ARCH
from . import cross_fixture as CF
from . import operator as OP
from . import per_fixture as PF
from . import special as SP


SHORT_NAMES = {
    "tadbench_trainticket_F11":  "TADBench F-11",
    "tadbench_trainticket_F11b": "TADBench F-11b",
    "tadbench_trainticket_F04":  "TADBench F-04",
    "tadbench_trainticket_F19":  "TADBench F-19",
    "illinois_socialnetwork":    "Illinois SocialNetwork",
    "aiops_challenge":           "AIOps Challenge KPI",
    "lo2":                       "LO2 OAuth2",
    "multidim_localization":     "MultiDim Localization",
    "deeptralog":                "DeepTraLog F-01",
    "defects4j":                 "Defects4J",
    "bugsinpy":                  "BugsInPy",
    "promise_defect_prediction": "PROMISE",
}

FIXTURE_ORDER = [
    "01_F11",        "02_F11b",   "03_F04",            "04_F19",
    "05_Illinois",   "06_AIOpsChallenge", "07_LO2",
    "08_MultiDim",   "09_DeepTraLog",
    "10_Defects4J",  "11_BugsInPy",       "12_PROMISE",
]


def main(run_dir: str):
    S.install()

    run_path = Path(run_dir)
    results = run_path / "results"
    figs_out = run_path / "figures-py"
    figs_out.mkdir(parents=True, exist_ok=True)

    arch_dir  = figs_out / "00_architecture"
    cross_dir = figs_out / "cross_fixture"
    op_dir    = figs_out / "operator"
    spec_dir  = figs_out / "special"
    arch_dir.mkdir(exist_ok=True)
    cross_dir.mkdir(exist_ok=True)
    op_dir.mkdir(exist_ok=True)
    spec_dir.mkdir(exist_ok=True)

    print("[render.py] Loading per-fixture JSON…")
    per_fixture: list[dict] = []
    for fname in FIXTURE_ORDER:
        path = results / f"{fname}.json"
        if not path.is_file():
            print(f"[render.py]   skip: {fname} (no JSON)")
            continue
        try:
            d = json.loads(path.read_text())
        except json.JSONDecodeError as e:
            print(f"[render.py]   error: {fname}: {e}")
            continue
        d["short_name"] = SHORT_NAMES.get(d["manifest_name"], d["manifest_name"])
        d["fname"] = fname
        per_fixture.append(d)
    print(f"[render.py]   loaded {len(per_fixture)} fixtures")

    # ---- Architecture / infrastructure --------------------------
    print("[render.py] Rendering architecture figures…")
    ARCH.render_ml_vs_dsfb(arch_dir / "01_ml_vs_dsfb.png")
    ARCH.render_tier_breakdown(arch_dir / "02_tier_breakdown.png")
    bank_path = results / "bank.json"
    if bank_path.is_file():
        bank = json.loads(bank_path.read_text())
        ARCH.render_motif_affinity(bank["motifs"], arch_dir / "03_motif_affinity.png")
    else:
        print("[render.py]   warn: bank.json missing — skipping motif affinity figure")

    # ---- Cross-fixture summary ----------------------------------
    print("[render.py] Rendering cross-fixture figures…")
    CF.render_fusion_sweep(cross_dir / "01_fusion_sweep.png")
    CF.render_rscr_forest(per_fixture, cross_dir / "02_rscr_forest.png")
    CF.render_tier_firing_heatmap(per_fixture, cross_dir / "03_tier_firing.png")

    # ---- Operator figures (uses F-11 hardcoded packet) ---------
    print("[render.py] Rendering operator figures…")
    OP.render_evidence_cards(op_dir / "01_evidence_cards.png")
    OP.render_confuser_adjudication(op_dir / "02_confuser_adjudication.png")
    OP.render_anti_hallucination_ladder(op_dir / "03_anti_hallucination_ladder.png")

    # ---- Special figures -----------------------------------------
    print("[render.py] Rendering special figures…")
    f11 = next((f for f in per_fixture if f["fname"] == "01_F11"), None)
    if f11:
        SP.render_trace_event_collapse(f11, spec_dir / "01_trace_event_collapse.png")
        SP.render_theorem9_verification(f11, spec_dir / "02_theorem9_verification.png")

    # ---- Per-fixture figures ------------------------------------
    print(f"[render.py] Rendering per-fixture figures ({len(per_fixture)} fixtures)…")
    for f in per_fixture:
        out = figs_out / f["fname"]
        PF.render_per_fixture(f, out)

    print(f"[render.py] Done. Figures written to {figs_out}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(1)
    sys.exit(main(sys.argv[1]))
