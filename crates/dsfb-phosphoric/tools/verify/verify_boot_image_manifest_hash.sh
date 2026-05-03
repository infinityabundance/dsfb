#!/usr/bin/env bash
# G4 — verify_boot_image_manifest_hash
#
# Forensic role: image's compiled evidence **==** manifest-derived
# evidence. Per docs/TASK_SEAL_V0.md §6.1 the .pmanifest section carries
# 8 SHA-256 certificates (manifest_self_hash, compiler_image_hash, and
# six derived: effect_closure / capability_graph / mmio_range /
# ipc_route / stack_bound / loop_bound). recompute_certificates
# re-derives all 8 from the manifest source + program; the gate reads
# the .pmanifest section and asserts byte-for-byte match. Any single
# certificate mismatch → 1, with which one failed printed to stderr.
#
# Exit: 0 pass, 1 boundary violation, 2 scaffolding missing (warn-skip).

set -euo pipefail

if [ ! -x build/host-tools/recompute_certificates ] || [ ! -x build/host-tools/extract_pmanifest ]; then
    echo "[scaffold] recompute_certificates or extract_pmanifest not yet built"
    exit 2
fi

failed=0
for d in apps/*/; do
    manifest="${d}task.manifest.toml"
    image="${d}BOOTX64.EFI"
    if [ ! -f "$manifest" ] || [ ! -f "$image" ]; then
        continue
    fi
    expected="$(build/host-tools/recompute_certificates --manifest="$manifest" --image="$image")"
    actual="$(build/host-tools/extract_pmanifest --image="$image")"
    if [ "$expected" != "$actual" ]; then
        echo "G4 FAIL: $d .pmanifest certificate(s) diverge from re-derived"
        echo "  expected: $expected"
        echo "  actual:   $actual"
        failed=1
    fi
done

exit "$failed"
