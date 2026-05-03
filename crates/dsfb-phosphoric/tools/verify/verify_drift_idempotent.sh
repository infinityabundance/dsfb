#!/usr/bin/env bash
# verify_drift_idempotent.sh — chain-continuity gate for phosphoric_drift.
#
# Forensic role: classification is a *pure function* over the captured
# residuals. Same incident, same manifest, same classifier → byte-
# identical JSON verdict. Any divergence between two runs means the
# classifier is non-deterministic, which destroys the forensic claim.
#
# Run phosphoric_drift twice on the same .pfi; demand the JSON outputs
# diff to nothing.
#
# Exit: 0 pass, 1 idempotency violation, 2 scaffolding missing.

set -euo pipefail

if [ ! -x build/host-tools/phosphoric_drift ]; then
    echo "[scaffold] tools/phosphoric-host/phosphoric_drift.phos not yet built"
    exit 2
fi

failed=0
for pfi in apps/*/*.pfi tests/incidents/*.pfi; do
    if [ ! -f "$pfi" ]; then
        continue
    fi
    a="$(mktemp)"
    b="$(mktemp)"
    trap 'rm -f "$a" "$b"' EXIT
    build/host-tools/phosphoric_drift "$pfi" --json >"$a"
    build/host-tools/phosphoric_drift "$pfi" --json >"$b"
    if ! diff -q "$a" "$b" >/dev/null; then
        echo "verify_drift_idempotent FAIL: $pfi yields divergent JSON across runs"
        failed=1
    fi
    rm -f "$a" "$b"
done

exit "$failed"
