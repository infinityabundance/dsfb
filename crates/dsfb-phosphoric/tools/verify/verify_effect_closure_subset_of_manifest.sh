#!/usr/bin/env bash
# G2 — verify_effect_closure_subset_of_manifest
#
# Forensic role: nothing was *compileable* outside the manifest's effect
# alphabet. Per docs/TASK_SEAL_V0.md §4, M-003/M-004 are emitted by
# pcc when a function's transitive effect closure carries a bit not in
# [effects].allowed. This gate walks every apps/<task>/ that has a
# task.manifest.toml and confirms pcc accepts the program with the
# manifest in place. Any non-zero exit → boundary violation.
#
# Exit: 0 pass, 1 boundary violation, 2 scaffolding missing (warn-skip).

set -euo pipefail

if [ ! -x build/host-tools/pcc ]; then
    echo "[scaffold] compiler/pcc.phos not yet built as host binary"
    exit 2
fi

failed=0
for d in apps/*/; do
    manifest="${d}task.manifest.toml"
    if [ ! -f "$manifest" ]; then
        continue
    fi
    main="${d}main.phos"
    if [ ! -f "$main" ]; then
        continue
    fi
    if ! build/host-tools/pcc --manifest="$manifest" --emit-effect-closure "$main"; then
        echo "G2 FAIL: $d effect closure exceeds manifest bound (M-003/M-004)"
        failed=1
    fi
done

exit "$failed"
