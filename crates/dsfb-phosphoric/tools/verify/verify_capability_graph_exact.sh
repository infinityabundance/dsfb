#!/usr/bin/env bash
# G3 — verify_capability_graph_exact
#
# Forensic role: declared graph **==** compiled graph **==** boot graph
# **==** observed graph. EQUALITY, not subset — extras OR missing both
# fail. The strongest gate; subset would permit silent narrowing the
# manifest does not sanction.
#
# For each app that has a manifest, pcc emits caps.observed.json (the
# compiled-side authority graph). diff_caps_exact compares it byte-for-byte
# against the manifest's [[capabilities.entry]] set, with deterministic
# ordering. Any extras OR missing → 1.
#
# Exit: 0 pass, 1 boundary violation, 2 scaffolding missing (warn-skip).

set -euo pipefail

if [ ! -x build/host-tools/pcc ] || [ ! -x build/host-tools/diff_caps_exact ]; then
    echo "[scaffold] pcc or diff_caps_exact not yet built"
    exit 2
fi

failed=0
for d in apps/*/; do
    manifest="${d}task.manifest.toml"
    main="${d}main.phos"
    if [ ! -f "$manifest" ] || [ ! -f "$main" ]; then
        continue
    fi
    obs="${d}caps.observed.json"
    build/host-tools/pcc --manifest="$manifest" --emit-caps-observed="$obs" "$main" >/dev/null
    if ! build/host-tools/diff_caps_exact --manifest="$manifest" --observed="$obs"; then
        echo "G3 FAIL: $d capability graph diverges from manifest (M-011-equivalent)"
        failed=1
    fi
done

exit "$failed"
