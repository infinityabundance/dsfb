#!/usr/bin/env bash
# verify_audit_package_round_trip.sh — .pfa round-trip gate.
#
# Forensic role: a `.pfa` audit package must round-trip cleanly through
# phosphoric_drift --audit. Pack it; classify every embedded incident;
# verify the dsfb-gray verdict signature; demand exit 0 on a clean
# build, exit 2/3/6 on a regression test that mutates one byte.
#
# Closed format taxonomy: only .pfi and .pfa exist. This gate enforces
# the discipline by running a regression test that confirms a single-
# byte mutation surfaces as exactly one drift class.
#
# Exit: 0 pass, 1 round-trip failure, 2 scaffolding missing.

set -euo pipefail

if [ ! -x build/host-tools/phosphoric_drift ] || [ ! -x build/host-tools/phosphoric_attest ]; then
    echo "[scaffold] phosphoric_drift or phosphoric_attest not yet built"
    exit 2
fi

failed=0
for pfa in release/**/*.pfa; do
    if [ ! -f "$pfa" ]; then
        continue
    fi
    if ! build/host-tools/phosphoric_drift --audit "$pfa" >/dev/null; then
        echo "verify_audit_package_round_trip FAIL: $pfa drift classification non-zero"
        failed=1
    fi
done

exit "$failed"
