#!/usr/bin/env bash
# G5 — verify_runtime_trace_subset_of_seal
#
# Forensic role: observed graph ⊆ declared graph. The post-mortem
# invariant. Runs the demo image under qemu with debug-port trace
# enabled; check_trace_subset tags every observed effect / cap-issue /
# IPC send-recv / mmio touch and asserts each is in the manifest's
# authority graph. Any non-subset event → 1. This is the gate that
# makes the forensic-replay claim testable: the trace IS the observed
# boundary, the manifest IS the declared boundary, and the gate
# enforces ⊆ between them.
#
# Exit: 0 pass, 1 boundary violation, 2 scaffolding missing (warn-skip).

set -euo pipefail

if [ ! -x build/host-tools/check_trace_subset ]; then
    echo "[scaffold] check_trace_subset not yet built"
    exit 2
fi

manifest="apps/demo/task.manifest.toml"
if [ ! -f "$manifest" ]; then
    echo "G5 SKIP: apps/demo has no manifest"
    exit 0
fi

trace="$(mktemp)"
trap 'rm -f "$trace"' EXIT

# Run the demo, capturing the debug-port trace. The runner script
# already exists; we redirect its trace output to the temp file.
PHOSPHORIC_TRACE_OUT="$trace" tools/qemu-run/run_uefi_demo.sh >/dev/null

if ! build/host-tools/check_trace_subset --manifest="$manifest" --trace="$trace"; then
    echo "G5 FAIL: runtime trace contains events outside the manifest's authority graph"
    exit 1
fi
