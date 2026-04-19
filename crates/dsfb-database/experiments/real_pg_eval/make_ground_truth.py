#!/usr/bin/env python3
"""Emit a per-replication ground-truth JSON.

Expected env vars:
  FAULT              fault class name (for tracing)
  FAULT_MOTIF        motif classified by this fault
  FAULT_DESCRIPTION  prose description
  CHANNELS_RAW       newline-separated channels from fault_channels()
  TAPE_SHA           sha256 of the captured live tape
  GT_OUT             output path for ground_truth.json
"""
import json
import os
import sys
from pathlib import Path


def main():
    motif = os.environ["FAULT_MOTIF"]
    description = os.environ["FAULT_DESCRIPTION"]
    raw = os.environ.get("CHANNELS_RAW", "")
    tape_sha = os.environ["TAPE_SHA"]
    out_path = Path(os.environ["GT_OUT"])

    channels = []
    for line in raw.splitlines():
        ch = line.strip()
        if ch and ch not in channels:
            channels.append(ch)

    windows = []
    for ch in channels:
        windows.append({
            "motif": motif,
            "channel": ch,
            "t_start": 30.0,
            "t_end": 70.0,
        })

    gt = {
        "tape_sha256": tape_sha,
        "fault_description": description,
        "windows": windows,
        "notes": (
            "Channels are md5(queryid::text) from pg_stat_statements, "
            "resolved per-replication because pgbench -i regenerates "
            "pgbench tables with fresh OIDs. Window runs from fault "
            "injection (t=30 s) to end of capture (t=70 s); the "
            "detector's observed episode dwell is always shorter "
            "because the motif grammar caps episode length at the "
            "per-motif dwell cap in spec/motifs.yaml. TTD is measured "
            "against t_start=30 s."
        ),
    }
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w") as f:
        json.dump(gt, f, indent=2)
    print(f"wrote {out_path} ({len(windows)} windows)", file=sys.stderr)


if __name__ == "__main__":
    main()
