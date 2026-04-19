#!/usr/bin/env python3
"""Cold-start ablation (Pass-2 N2).

Quantifies the §43 (cold-start motif tuning gap) limitation: how much
TTD does each detector lose when the observer is started near the
fault, instead of after a full warmup window?

Operationally: take each (fault, replication) tape under
`experiments/real_pg_eval/out/`, drop the leading N seconds of
residual samples, repackage as a fresh tape with a recomputed
SHA-256 sidecar, write a matching ground-truth JSON whose
`tape_sha256` references the truncated tape, then call
`replay_tape_baselines` on the (truncated_tape, truncated_gt) pair.
The resulting bakeoff CSV's `ttd_mean_s` column is the cold-start
TTD for that (fault, rep, detector, warmup_s).

Output:
  out/cold_start.csv with columns:
    fault, rep, detector, warmup_seconds, ttd_mean_s, recall, far_per_hour

The §43 paragraph in paper/dsfb-database.tex cites the
warmup_seconds = 0 vs warmup_seconds = 30 delta on DSFB's
plan_regression_onset detection.

Determinism. The truncation + rehash is a pure function of the
original tape and the warmup. `replay_tape_baselines` is a pure
function of (tape, ground_truth). Two independent runs produce
byte-equal cold_start.csv.
"""

import argparse
import csv
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


GT_FOR_FAULT = {
    "drop_constraint": "plan_regression_onset",
    "stats_stale": "cardinality_mismatch_regime",
    "lock_hold": "contention_ramp",
    "cache_evict": "cache_collapse",
}


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--pg-out", required=True,
                   help="Path to experiments/real_pg_eval/out/")
    p.add_argument("--bin", required=True,
                   help="Path to the replay_tape_baselines binary.")
    p.add_argument("--out", required=True,
                   help="Output dir; cold_start.csv lands here.")
    p.add_argument("--warmups", default="0,10,20,30",
                   help="Comma-separated warmup-truncation seconds.")
    p.add_argument("--max-reps", type=int, default=10,
                   help="Cap reps per fault for fast smoke runs.")
    return p.parse_args()


def truncate_tape(src_jsonl: Path, src_hash: Path, dst_jsonl: Path, warmup_s: float) -> str:
    """Drop the first warmup_s seconds of samples; rewrite JSON manifest.

    Returns the new tape SHA-256 hex string. The manifest schema mirrors
    `dsfb_database::live::tape::TapeManifest` (sha256, sample_count,
    first_t, last_t, crate_version, source) so that
    `replay_tape_baselines` can verify the truncated tape via
    `tape::load_and_verify`.
    """
    src_manifest = json.loads(src_hash.read_text()) if src_hash.exists() else {}
    h = hashlib.sha256()
    sample_count = 0
    first_t = None
    last_t = None
    with src_jsonl.open() as f, dst_jsonl.open("wb") as g:
        for line in f:
            if not line.strip():
                continue
            sample = json.loads(line)
            t = float(sample.get("t", 0.0))
            if t < warmup_s:
                continue
            line_bytes = (line.rstrip("\n") + "\n").encode("utf-8")
            g.write(line_bytes)
            h.update(line_bytes)
            sample_count += 1
            first_t = t if first_t is None else min(first_t, t)
            last_t = t if last_t is None else max(last_t, t)
    digest = h.hexdigest()
    sidecar = dst_jsonl.with_suffix(dst_jsonl.suffix + ".hash")
    manifest = {
        "sha256": digest,
        "sample_count": sample_count,
        "first_t": first_t,
        "last_t": last_t,
        "crate_version": src_manifest.get("crate_version", "0.1.0"),
        "source": src_manifest.get("source", "cold_start truncation") + f" (warmup={warmup_s}s)",
    }
    sidecar.write_text(json.dumps(manifest, indent=2))
    return digest


def patch_ground_truth(src_gt: Path, dst_gt: Path, new_tape_hash: str):
    gt = json.loads(src_gt.read_text())
    gt["tape_sha256"] = new_tape_hash
    dst_gt.write_text(json.dumps(gt, indent=2))


def parse_bakeoff(bakeoff_csv: Path):
    """Returns list of dicts indexed by detector x motif."""
    rows = []
    with bakeoff_csv.open() as f:
        lines = [l for l in f if not l.startswith("#")]
    for r in csv.DictReader(lines):
        rows.append(r)
    return rows


def main():
    a = parse_args()
    pg_out = Path(a.pg_out)
    out_dir = Path(a.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    warmups = [float(s) for s in a.warmups.split(",") if s.strip()]

    out_csv = out_dir / "cold_start.csv"
    fields = ["fault", "rep", "detector", "motif", "warmup_seconds",
              "ttd_median_s", "ttd_p95_s", "recall", "precision", "f1", "tp", "fp", "fn"]
    with out_csv.open("w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        for fault in sorted(GT_FOR_FAULT):
            fault_dir = pg_out / fault
            if not fault_dir.is_dir():
                continue
            reps = sorted(fault_dir.glob("r*"))[: a.max_reps]
            for rep_dir in reps:
                tape = rep_dir / "live.tape.jsonl"
                tape_hash = rep_dir / "live.tape.jsonl.hash"
                gt = rep_dir / "ground_truth.json"
                if not tape.exists() or not gt.exists():
                    continue
                gt_motif = GT_FOR_FAULT[fault]
                for warm in warmups:
                    with tempfile.TemporaryDirectory() as tmp:
                        tmp = Path(tmp)
                        new_tape = tmp / "tape.jsonl"
                        new_gt = tmp / "ground_truth.json"
                        new_hash = truncate_tape(tape, tape_hash, new_tape, warm)
                        patch_ground_truth(gt, new_gt, new_hash)
                        out_sub = tmp / "bakeoff_out"
                        try:
                            subprocess.run(
                                [a.bin, "--tape", str(new_tape),
                                 "--ground-truth", str(new_gt),
                                 "--out", str(out_sub)],
                                check=True, capture_output=True,
                            )
                        except subprocess.CalledProcessError as e:
                            print(f"replay failed on {fault}/{rep_dir.name}"
                                  f" warmup={warm}s: {e.stderr.decode()[:300]}",
                                  file=sys.stderr)
                            continue
                        bakeoff = out_sub / "bakeoff.csv"
                        if not bakeoff.exists():
                            continue
                        for r in parse_bakeoff(bakeoff):
                            if r["motif"] != gt_motif:
                                continue
                            w.writerow({
                                "fault": fault,
                                "rep": rep_dir.name,
                                "detector": r["detector"],
                                "motif": r["motif"],
                                "warmup_seconds": warm,
                                "ttd_median_s": r.get("ttd_median_s", ""),
                                "ttd_p95_s": r.get("ttd_p95_s", ""),
                                "recall": r.get("recall", ""),
                                "precision": r.get("precision", ""),
                                "f1": r.get("f1", ""),
                                "tp": r.get("tp", ""),
                                "fp": r.get("fp", ""),
                                "fn": r.get("fn", ""),
                            })
    print(f"wrote {out_csv}")


if __name__ == "__main__":
    main()
