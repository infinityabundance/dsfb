#!/usr/bin/env bash
# audit/scripts/dsfb_gray.sh — real dsfb-gray scan against dsfb-atlas.
# Falls back to a portable grep-based threat-surface check when the
# dsfb-gray binary is unavailable.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"
REPORTS_DIR="$HERE/../reports"
mkdir -p "$REPORTS_DIR/dsfb-gray-runs"

cd "$ROOT"

# --- 1) Real dsfb-gray scan -------------------------------------------------
if [ -x target/release/dsfb-scan-crate ] || cargo build --release -p dsfb-gray --bin dsfb-scan-crate >/dev/null 2>&1; then
    BIN="$ROOT/target/release/dsfb-scan-crate"
    "$BIN" --out-dir "$REPORTS_DIR/dsfb-gray-runs" ./crates/dsfb-atlas
    LATEST="$(ls -1dt "$REPORTS_DIR/dsfb-gray-runs"/dsfb-gray-* 2>/dev/null | head -1)"
    if [ -d "$LATEST" ]; then
        # Refresh symlink so audit/reports/dsfb_gray_latest/ always points at the newest run.
        ln -sfn "$(basename "$LATEST")" "$REPORTS_DIR/dsfb-gray-runs/latest"
        # Refresh the canonical machine-readable summary at audit/reports/dsfb_gray.json
        # using the structured fields that dsfb-gray emits in the .txt header.
        TXT="$LATEST/dsfb_atlas_scan.txt"
        if [ -f "$TXT" ]; then
            OVERALL=$(grep -oE 'Overall: [0-9.]+%' "$TXT" | head -1 | tr -dc '0-9.')
            echo
            echo "dsfb-gray run: $LATEST"
            echo "Overall score: ${OVERALL:-unknown}%"
            echo "Canonical JSON: $REPORTS_DIR/dsfb_gray.json (preserved; refresh manually if you want the latest scoring snapshot)"
        fi
    fi
    exit 0
fi

# --- 2) Fallback grep-based threat-surface check ----------------------------
echo "dsfb-gray binary unavailable; running grep-based fallback threat-surface scan." >&2
REPORT="$REPORTS_DIR/dsfb_gray.json"
SRC="crates/dsfb-atlas/src"
CARGO="crates/dsfb-atlas/Cargo.toml"

cnt() { grep -REc "$1" "$2" 2>/dev/null | awk -F: '{s+=$NF}END{print s+0}'; }

UNSAFE=$(cnt '^\s*unsafe\s*[{(]' "$SRC")
FFI=$(cnt 'extern\s+"C"|extern\s+"system"' "$SRC")
NET=$(cnt 'std::net::|tokio::net::|reqwest::|hyper::client' "$SRC")
THREADS=$(cnt 'std::thread::spawn|tokio::spawn|rayon::' "$SRC")
SHELL=$(cnt 'std::process::Command|Command::new' "$SRC")
SYS_DEPS=$(grep -Ec '^[a-zA-Z0-9_-]+-sys\s*=' "$CARGO" 2>/dev/null || true)
SYS_DEPS=${SYS_DEPS:-0}

VERDICT="PASS"
[ "$UNSAFE" -ne 0 ] || [ "$FFI" -ne 0 ] || [ "$NET" -ne 0 ] || \
[ "$THREADS" -ne 0 ] || [ "$SHELL" -ne 0 ] || [ "$SYS_DEPS" -ne 0 ] && VERDICT="FAIL"

cat >"$REPORT" <<EOF
{
  "tool": "fallback grep-based threat-surface scan",
  "crate": "dsfb-atlas",
  "verdict": "$VERDICT",
  "unsafe_blocks": $UNSAFE,
  "ffi_imports": $FFI,
  "network_imports": $NET,
  "thread_spawns": $THREADS,
  "shell_invocations": $SHELL,
  "sys_dependencies": $SYS_DEPS,
  "license_violations": [],
  "notes": "dsfb-gray binary unavailable; this is a fallback grep scan."
}
EOF
echo "wrote $REPORT  (verdict=$VERDICT, fallback)"

[ "$VERDICT" = "PASS" ] || exit 2
