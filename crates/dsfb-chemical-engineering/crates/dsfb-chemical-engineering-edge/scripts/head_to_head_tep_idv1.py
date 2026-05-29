#!/usr/bin/env python3
"""P37: head-to-head worked example on Tennessee Eastman IDV-1 (committed slice).

This is a *demonstration of the recorded artifact*, NOT a detection-superiority claim. It contrasts what
two pipelines hand an operator from the SAME committed input slice
(`data/slices/tennessee_eastman_idv01.csv`):

  INCUMBENT practice          -- a bank of raw threshold detectors + an MSPC contribution plot:
      * raw breach-steps       (every detector-sample over threshold; the "everything is red" picture)
      * ISA-18.2 annunciations (false->true rising edges, i.e. how many times an annunciator would fire)
      * contribution-plot top variables for the sustained fault (a ranked list, no disposition, no record)

  DSFB Chemical Court Record  -- a fused, hash-linked, replay-verified bundle:
      * fused episodes + the breach-step -> episode compression ratio (triage reduction)
      * per-episode admissibility badges (CANDIDATE_FAULT vs STRUCTURE_ONLY)
      * the count of *rejected* candidates, each logged WITH a rejection reason + evidence hash
        (the silence is itself recorded -- the auditability delta)
      * global badges, evidence_root, bundle_root, replay_deterministic

Everything below is read from the deterministic run outputs; nothing is hand-entered. The run outputs are
gitignored (regenerable); this recipe is committed so the paper / docs numbers are reproducible.

Reproduce (deterministic, replay_deterministic=true):
    cargo run --release -p dsfb-chemical-engineering-edge -- demo
    cargo run --release -p dsfb-chemical-engineering-edge -- casefile tennessee_eastman_idv01
    python3 scripts/head_to_head_tep_idv1.py

RUN_DIR      defaults to the newest output-dsfb-chemical-engineering/<stamp>/ under the workspace root.
CASEFILE_DIR defaults to output-dsfb-chemical-engineering-casefile/tennessee_eastman_idv01/.
"""
import csv
import glob
import json
import os
from collections import defaultdict

DATASET = "tennessee_eastman_idv01"
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))  # the edge crate dir
# workspace root holds output-dsfb-chemical-engineering/: it is two levels up (../crates/<crate>)
WS = os.path.dirname(os.path.dirname(HERE))


def newest_run_dir():
    env = os.environ.get("RUN_DIR")
    if env:
        return env
    cands = sorted(glob.glob(os.path.join(WS, "output-dsfb-chemical-engineering", "2*")))
    if not cands:
        raise SystemExit("no run dir found; run `-- demo` first or set RUN_DIR")
    return cands[-1]


def casefile_dir(run_dir):
    env = os.environ.get("CASEFILE_DIR")
    if env:
        return env
    # prefer the dedicated casefile output; fall back to the per-dataset bundle inside the run dir
    a = os.path.join(WS, "output-dsfb-chemical-engineering-casefile", DATASET,
                      "dsfb_chemical_engineering_casefile_v1")
    b = os.path.join(run_dir, "datasets", DATASET, "dsfb_chemical_engineering_casefile_v1")
    return a if os.path.exists(a) else b


def read_metrics(run_dir):
    with open(os.path.join(run_dir, "metrics.csv")) as f:
        for row in csv.DictReader(f):
            if row["dataset"] == DATASET:
                return row
    raise SystemExit(f"{DATASET} not in metrics.csv")


def incumbent_alarms(run_dir):
    """Breach-steps + ISA-18.2 alarm activations (rising edges) from the raw detector bank."""
    path = os.path.join(run_dir, "datasets", DATASET, "detector_outputs.csv")
    byd = defaultdict(list)
    with open(path) as f:
        for r in csv.DictReader(f):
            byd[r["detector_id"]].append((int(r["time_index"]), r["breach"].strip().lower() == "true"))
    breach_steps = activations = 0
    n = 0
    for seq in byd.values():
        seq.sort()
        b = [x[1] for x in seq]
        n = len(b)
        breach_steps += sum(b)
        activations += sum(1 for i in range(len(b)) if b[i] and (i == 0 or not b[i - 1]))
    return breach_steps, activations, len(byd), n


def candidate_fault_episode(cf):
    """Return (start,end) of the admitted CANDIDATE_FAULT episode (the sustained fault) from the casefile."""
    for e in cf.get("episode_badges", []):
        if e.get("badge") == "CANDIDATE_FAULT":
            return e["start_index"], e["end_index"]
    return None


def contribution_top(run_dir, span, k=5):
    path = os.path.join(run_dir, "datasets", DATASET, "contribution_traces.csv")
    if span is None or not os.path.exists(path):
        return []
    s, e = span
    out = []
    with open(path) as f:
        for r in csv.DictReader(f):
            if int(r["episode_start"]) == s and int(r["episode_end"]) == e and int(r["rank"]) <= k:
                out.append((int(r["rank"]), r["variable"], float(r["mean_abs_z"])))
    return sorted(out)


def main():
    run_dir = newest_run_dir()
    cf_dir = casefile_dir(run_dir)
    m = read_metrics(run_dir)
    cf = json.load(open(os.path.join(cf_dir, "casefile.json")))

    breach_steps, activations, n_det, n_samp = incumbent_alarms(run_dir)
    span = candidate_fault_episode(cf)
    top = contribution_top(run_dir, span)
    fused = int(m["fused_episodes"])
    counts = cf.get("counts", {})

    print(f"# Head-to-head worked example -- {DATASET}")
    print(f"run_dir = {os.path.relpath(run_dir, WS)}")
    print(f"input   = data/slices/{DATASET}.csv  ({m['n_samples']} samples x {m['n_vars']} vars, "
          f"kind={m['kind']})\n")

    print("## Incumbent practice (raw detector bank + MSPC contribution plot)")
    print(f"  detectors                 : {n_det}")
    print(f"  raw breach-steps          : {breach_steps}  "
          f"(avg {breach_steps / n_samp:.2f} detectors simultaneously in alarm per sample)")
    print(f"  ISA-18.2 alarm activations: {activations}  (false->true rising edges, summed over detectors)")
    if top:
        s, e = span
        print(f"  contribution plot @ sustained fault (steps {s}-{e}): "
              + ", ".join(f"{v}" for _r, v, _z in top))
    print("  -> output is transient: scores + a ranked plot. Nothing persisted, replayable, or admitted.\n")

    print("## DSFB Chemical Court Record (fused, hash-linked, replay-verified bundle)")
    print(f"  fused episodes            : {fused}")
    print(f"  triage reduction          : {breach_steps} breach-steps -> {fused} episodes "
          f"= {breach_steps / fused:.1f}x  ({activations} activations -> {fused} = {activations / fused:.1f}x)")
    cand = counts.get("candidate_fault", 0)
    print(f"  admitted badges           : {cand} CANDIDATE_FAULT + {fused - cand} STRUCTURE_ONLY")
    print(f"  rejected candidates       : {counts.get('rejected_candidates', '?')} "
          f"(each logged WITH a rejection reason + evidence hash -- the silence is recorded)")
    print(f"  global badges             : {', '.join(cf.get('global_badges', []))}")
    print(f"  evidence_root             : {cf.get('evidence_root')}")
    print(f"  bundle_root               : {cf.get('bundle_root')}")
    print(f"  replay_deterministic      : {cf.get('replay_deterministic')}")
    print(f"  key_statement             : {cf.get('key_statement')}")


if __name__ == "__main__":
    main()
