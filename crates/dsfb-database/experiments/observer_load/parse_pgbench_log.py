#!/usr/bin/env python3
"""Parse a pgbench per-transaction log and emit one CSV row.

pgbench -l writes one line per transaction with columns
  client_id transaction_no time_us script_no time_epoch time_us

Columns vary slightly across versions; transaction latency in
microseconds is column 3 (index 2, 0-based) in PG 15-17.

Usage: parse_pgbench_log.py <raw_log> <condition> <rep>
Emits one CSV line to stdout:
  condition,rep,n_tx,p50_us,p95_us,p99_us,p99_9_us
"""
import bisect
import sys
from pathlib import Path


def percentile(sorted_vals, p):
    if not sorted_vals:
        return 0.0
    n = len(sorted_vals)
    k = (n - 1) * (p / 100.0)
    lo = int(k)
    hi = min(lo + 1, n - 1)
    frac = k - lo
    return sorted_vals[lo] * (1 - frac) + sorted_vals[hi] * frac


def main():
    raw = Path(sys.argv[1])
    condition = sys.argv[2]
    rep = sys.argv[3]
    if not raw.exists():
        print(f"{condition},{rep},0,0,0,0,0")
        return
    lats = []
    with raw.open() as f:
        for line in f:
            parts = line.strip().split()
            if len(parts) < 3:
                continue
            try:
                lats.append(int(parts[2]))
            except ValueError:
                continue
    lats.sort()
    n = len(lats)
    p50 = percentile(lats, 50)
    p95 = percentile(lats, 95)
    p99 = percentile(lats, 99)
    p99_9 = percentile(lats, 99.9)
    print(f"{condition},{rep},{n},{p50:.0f},{p95:.0f},{p99:.0f},{p99_9:.0f}")


if __name__ == "__main__":
    main()
