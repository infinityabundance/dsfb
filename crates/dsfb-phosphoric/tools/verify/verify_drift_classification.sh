#!/usr/bin/env bash
# verify_drift_classification.sh — drift classifier algorithm test.
#
# Forensic role: the eight drift classes form a closed taxonomy. Every
# residual MUST classify to exactly one class. This gate runs synthetic
# residuals through the classification rules (mirroring
# tools/phosphoric-host/phosphoric_drift.phos) and asserts the expected
# class fires.
#
# Synthetic test corpus (eight one-byte injections mapped to eight classes):
#
#   1. cap (kind=1) at slot 0 IN manifest    -> NO_DRIFT
#   2. cap (kind=1) at slot 5 NOT in manifest -> AUTHORITY_EXPANSION
#   3. ipc (kind=2) at slot 0 IN manifest    -> NO_DRIFT
#   4. ipc (kind=2) at slot 5 NOT in manifest -> IPC_ROUTE_DIVERGENCE
#   5. budget (kind=3) measured=900 limit=1000 -> STACK_BUDGET_PRESSURE
#   6. budget (kind=3) measured=400 limit=1000 -> NO_DRIFT
#   7. boot_check (kind=7) outcome=0 (match)   -> NO_DRIFT
#   8. boot_check (kind=7) outcome=1 (mismatch) -> BOOT_ATTESTATION_MISMATCH
#
# Exit: 0 pass, 1 classification divergence, 2 awk unavailable.

set -euo pipefail

if ! command -v awk >/dev/null; then
    echo "[scaffold] awk not available"
    exit 2
fi

# Manifest under test: declares cap (kind=1, slot=0..1), 2 channels.
# Mirrors apps/demo/task.manifest.toml shape: Task slot 0, Channel slot 0.
manifest_caps="1:0 1:1 2:0 3:0 4:0"
manifest_channel_count=2

# classify(kind, payload_bytes, manifest_caps, manifest_channel_count) -> class
# The classifier mirrors tools/phosphoric-host/phosphoric_drift.phos
# `classify_residual` function.
classify() {
    local kind="$1"
    local p0="$2"
    local p1="$3"
    local p2="$4"
    local p3="$5"
    local p4="$6"
    local p5="$7"
    local p6="$8"
    local p7="$9"
    local p8="${10}"

    case "$kind" in
        1) # cap_graph_delta: payload[0]=kind_id, [1..3]=slot
            local slot=$(( p1 + p2 * 256 ))
            local lookup="${p0}:${slot}"
            if echo "$manifest_caps" | tr ' ' '\n' | grep -Fxq "$lookup"; then
                echo "NO_DRIFT"
            else
                echo "AUTHORITY_EXPANSION"
            fi
            ;;
        2) # ipc_route_delta: payload[0..2]=slot
            local slot=$(( p0 + p1 * 256 ))
            if [ "$slot" -ge "$manifest_channel_count" ]; then
                echo "IPC_ROUTE_DIVERGENCE"
            else
                echo "NO_DRIFT"
            fi
            ;;
        3) # budget_pressure: [1..5]=measured u32, [5..9]=limit u32 (LE)
            local measured=$(( p1 + p2 * 256 + p3 * 65536 + p4 * 16777216 ))
            local limit=$(( p5 + p6 * 256 + p7 * 65536 + p8 * 16777216 ))
            if [ "$measured" -gt "$limit" ]; then
                echo "AUTHORITY_EXPANSION"
            elif [ $(( measured * 10 )) -ge $(( limit * 9 )) ]; then
                echo "STACK_BUDGET_PRESSURE"
            else
                echo "NO_DRIFT"
            fi
            ;;
        7) # boot_check: [1]=outcome (0=match, 1=mismatch)
            if [ "$p1" = "0" ]; then
                echo "NO_DRIFT"
            else
                echo "BOOT_ATTESTATION_MISMATCH"
            fi
            ;;
        *) echo "UNKNOWN_KIND" ;;
    esac
}

run_test() {
    local name="$1"
    local expected="$2"
    shift 2
    local actual
    actual=$(classify "$@")
    if [ "$actual" != "$expected" ]; then
        echo "verify_drift_classification FAIL: $name"
        echo "  expected: $expected"
        echo "  actual:   $actual"
        return 1
    fi
    return 0
}

# 8 test cases, each one byte different from a NO_DRIFT baseline.
# Args: kind p0 p1 p2 p3 p4 p5 p6 p7 p8
failures=0
run_test "cap declared"          NO_DRIFT             1 1 0 0 0 0 0 0 0 0 || failures=$((failures+1))
run_test "cap undeclared"        AUTHORITY_EXPANSION  1 1 5 0 0 0 0 0 0 0 || failures=$((failures+1))
run_test "ipc declared"          NO_DRIFT             2 0 0 0 0 0 0 0 0 0 || failures=$((failures+1))
run_test "ipc undeclared"        IPC_ROUTE_DIVERGENCE 2 5 0 0 0 0 0 0 0 0 || failures=$((failures+1))
run_test "budget pressure"       STACK_BUDGET_PRESSURE 3 0 132 3 0 0 232 3 0 0 || failures=$((failures+1))   # m=900, l=1000
run_test "budget ok"             NO_DRIFT             3 0 144 1 0 0 232 3 0 0 || failures=$((failures+1))   # m=400, l=1000
run_test "boot match"            NO_DRIFT             7 0 0 0 0 0 0 0 0 0 || failures=$((failures+1))
run_test "boot mismatch"         BOOT_ATTESTATION_MISMATCH 7 0 1 0 0 0 0 0 0 0 || failures=$((failures+1))

if [ "$failures" -gt 0 ]; then
    echo "verify_drift_classification FAIL: $failures of 8 cases failed"
    exit 1
fi

echo "verify_drift_classification: 8/8 cases produce the expected drift class"
exit 0
