#!/usr/bin/env bash
# check_manifest_doctrine.sh — manifest schema doctrine immutability gate.
#
# Third in the doctrine-gate trilogy:
#   - check_classifier_doctrine.sh  → goal #4 (forensic classification)
#   - check_residual_doctrine.sh    → goal #3 (residual truth)
#   - check_manifest_doctrine.sh    → goal #2 (task-seal closure)  [this]
#
# Apex framing (FORENSIC_PRIMACY.md, memory/feedback_residual_court.md):
#
#   "Phosphoric is a drift-and-slew residual court, not a log analyzer."
#
# `compiler/manifest.phos` is the host-profile schema reader and
# validator: it defines the closed sets that the task-seal contract
# rests on. `kernel/manifest.phos` is the runtime-profile mirror.
#
# This gate fails closed if any of those closed sets drift:
#   - the 10 forbidden keys (heap/filesystem/network/...)
#   - the 4 manifest profiles (Boot/Host/Trusted/Runtime)
#   - the 4 capability kinds (Task=1, Channel=2, Window=3, Framebuffer=4)
#   - the M-001..M-012 diagnostic code namespace
#   - the host/runtime field-name agreement on shared struct keys
#   - log-analyzer vocabulary leaking into the schema source
#
# It does NOT verify the schema is wired up at runtime (no producer
# integration check; that belongs in verify-manifest-doctrine
# subgates). It ONLY verifies the doctrine in source.
#
# Exit 0: doctrine intact.
# Exit 1: doctrine violation (named).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

host_src="compiler/manifest.phos"
runtime_src="kernel/manifest.phos"

if [ ! -r "$host_src" ]; then
    echo "[manifest-doctrine] FAIL: host source not found at $host_src" >&2
    exit 1
fi
if [ ! -r "$runtime_src" ]; then
    echo "[manifest-doctrine] FAIL: runtime source not found at $runtime_src" >&2
    exit 1
fi

fails=0
note_fail() {
    fails=$((fails + 1))
    echo "[manifest-doctrine] FAIL: $*" >&2
}

# -------------------------------------------------------------------
# 1. The 10 forbidden keys. Per docs/TASK_SEAL_V0.md, every task
#    manifest must declare each of these keys explicitly set to
#    `false` — absence is not the same as `false`. The closed list
#    is encoded in the `ManifestForbiddenSeen` struct in the host
#    source. Drift in this list (rename, removal, addition) breaks
#    the task-seal completeness check (M-007).
# -------------------------------------------------------------------
forbidden_keys=(
    "heap"
    "filesystem"
    "network"
    "dynamic_loading"
    "ambient_mmio"
    "reflection"
    "pointer_types"
    "hidden_syscalls"
    "undeclared_ipc"
    "undeclared_effects"
)

for key in "${forbidden_keys[@]}"; do
    # Match as a struct field declaration: `^    KEY: bool,?` (allowing
    # the indent and optional trailing comma).
    if ! grep -q -E "^\s*$key:\s*bool" "$host_src"; then
        note_fail "forbidden key '$key' missing from ManifestForbiddenSeen in $host_src"
    fi
done

# -------------------------------------------------------------------
# 2. The 4 manifest profiles. Closed enum. Adding a fifth profile is
#    a doctrine change (TASK_SEAL_V0.md §0.2) — gates fail until the
#    doctrine is updated.
# -------------------------------------------------------------------
required_profiles=(
    "Boot"
    "Host"
    "Trusted"
    "Runtime"
)

#
# Both the enum declaration line (bare `Variant,`) AND a qualified
# use (`ManifestProfile::Variant`) must be present. Requiring just one
# misses a partial rename where the declaration is renamed but use
# sites are left dangling (or vice versa) — the compile would reject
# but the doctrine gate should not need a compile to catch it.
for prof in "${required_profiles[@]}"; do
    if ! grep -q -E "ManifestProfile::$prof\b" "$host_src"; then
        note_fail "ManifestProfile use '${prof}' missing from $host_src (no qualified reference)"
    fi
    if ! grep -q -E "^\s*$prof,?\s*$" "$host_src"; then
        note_fail "ManifestProfile variant '$prof' missing from enum declaration in $host_src"
    fi
done

# -------------------------------------------------------------------
# 3. The 4 capability kinds with their numeric IDs. The (name, id)
#    pairing must appear in the comment table at the struct
#    declaration; both host and runtime sources document the table.
#    A drift in either side unmoors the host parser from the runtime
#    consumer.
# -------------------------------------------------------------------
cap_pairs=(
    "1=Task"
    "2=Channel"
    "3=Window"
    "4=Framebuffer"
)

for src in "$host_src" "$runtime_src"; do
    for pair in "${cap_pairs[@]}"; do
        code="${pair%%=*}"
        name="${pair#*=}"
        # The comment format in both sources is `// kind_id: u8  // 1=Task,
        # 2=Channel, 3=Window, 4=Framebuffer`. Match `N=Name` as one
        # contiguous fragment after stripping commas/whitespace.
        if ! grep -q -E "$code\s*=\s*$name" "$src"; then
            note_fail "capability kind pairing '$pair' missing from $src"
        fi
    done
done

# -------------------------------------------------------------------
# 4. The M-prefix diagnostic-code namespace (M-001..M-012). Each code
#    has a fixed semantic per the comment block at the top of the
#    host source. Removing a code, renaming it, or repurposing it
#    is a doctrine change.
# -------------------------------------------------------------------
m_codes=(
    "M-001"
    "M-002"
    "M-003"
    "M-004"
    "M-005"
    "M-006"
    "M-007"
    "M-008"
    "M-009"
    "M-010"
    "M-011"
    "M-012"
)

for code in "${m_codes[@]}"; do
    if ! grep -q -E "$code\b" "$host_src"; then
        note_fail "diagnostic code '$code' missing from $host_src"
    fi
done

# -------------------------------------------------------------------
# 5. Host/runtime struct-field agreement. A small set of fields must
#    exist on both sides with matching names and types so the boot
#    loader can serialize the host-parsed manifest and have the
#    runtime kernel materialize it without reinterpretation.
# -------------------------------------------------------------------
#
# `manifest_hash` is runtime-side; the host carries the richer pair
# `manifest_self_hash` + `compiler_image_hash` which the boot loader
# combines into the runtime's single `manifest_hash` field. So the
# hash-presence check is per-side, not strictly identical naming.
shared_fields=(
    "kind_id:\\s*u8"
    "slot:\\s*u16"
    "capacity:\\s*u16"
    "id_hash"
    "task_stack_limit:\\s*u32"
    "kernel_init_limit:\\s*u32"
    "loop_bound_max:\\s*u32"
    "residual_chain_every_n_events:\\s*u16"
    "residual_ring_size:\\s*u16"
)
shared_names=(
    "kind_id: u8"
    "slot: u16"
    "capacity: u16"
    "id_hash"
    "task_stack_limit: u32"
    "kernel_init_limit: u32"
    "loop_bound_max: u32"
    "residual_chain_every_n_events: u16"
    "residual_ring_size: u16"
)

for i in "${!shared_fields[@]}"; do
    pat="${shared_fields[$i]}"
    name="${shared_names[$i]}"
    for src in "$host_src" "$runtime_src"; do
        if ! grep -q -E "$pat" "$src"; then
            note_fail "shared field '$name' missing from $src"
        fi
    done
done

# Per-side hash discipline: host carries both manifest_self_hash and
# compiler_image_hash; runtime carries the combined manifest_hash.
if ! grep -q -E "manifest_self_hash" "$host_src"; then
    note_fail "host source missing 'manifest_self_hash' field"
fi
if ! grep -q -E "compiler_image_hash" "$host_src"; then
    note_fail "host source missing 'compiler_image_hash' field"
fi
if ! grep -q -E "manifest_hash" "$runtime_src"; then
    note_fail "runtime source missing 'manifest_hash' field"
fi

# -------------------------------------------------------------------
# 6. No log-analyzer vocabulary in either source. Same boundary
#    discipline as check_residual_doctrine.sh — non-alphanumeric
#    boundary so compound names (e.g. severity_level, anomaly_score)
#    are caught.
# -------------------------------------------------------------------
forbidden_terms=(
    "score"
    "maybe"
    "probably"
    "suspicious"
    "unusual"
    "anomaly"
    "heuristic"
    "probabilistic"
    "histogram"
    "telemetry"
    "log_event"
    "log_record"
    "observability"
    "severity"
)

for term in "${forbidden_terms[@]}"; do
    for src in "$host_src" "$runtime_src"; do
        while IFS= read -r line; do
            [ -z "$line" ] && continue
            note_fail "forbidden term '$term' in $src: $line"
        done < <(grep -n -i -E "(^|[^A-Za-z0-9])$term([^A-Za-z0-9]|$)" "$src" 2>/dev/null | grep -v -E "FORBIDDEN|MUST NOT|never (use|emit)|anti-pattern" || true)
    done
done

# -------------------------------------------------------------------
# 7. Apex framing. Both sources must name the artifact as a
#    'manifest' and reference the task-seal containment doctrine
#    (the host parser by talking about the schema; the runtime by
#    talking about containment).
# -------------------------------------------------------------------
if ! grep -q -i "\\bmanifest\\b" "$host_src"; then
    note_fail "host source does not reference 'manifest' as the primitive"
fi
if ! grep -q -i "\\bmanifest\\b" "$runtime_src"; then
    note_fail "runtime source does not reference 'manifest' as the primitive"
fi
if ! grep -q -i -E "containment|cap_in_manifest|channel_in_manifest" "$runtime_src"; then
    note_fail "runtime source does not frame the primitive as containment of authority"
fi

# -------------------------------------------------------------------
# Summary.
# -------------------------------------------------------------------
echo "============================================================"
echo "  Phosphoric manifest doctrine gate"
echo "  host source:    $host_src"
echo "  runtime source: $runtime_src"
echo "  framing: 'task-seal closure — declared authority is the bound'"
echo "============================================================"

if [ "$fails" -eq 0 ]; then
    echo "  [manifest-doctrine] OK — forbidden keys, profiles, cap kinds,"
    echo "                       M-prefix diagnostic codes, host/runtime"
    echo "                       field agreement, and vocabulary all match"
    echo "                       doctrine"
    exit 0
fi

echo "  [manifest-doctrine] $fails violation(s) — see FAIL lines above"
exit 1
