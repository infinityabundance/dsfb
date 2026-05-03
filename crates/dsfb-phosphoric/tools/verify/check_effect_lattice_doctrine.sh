#!/usr/bin/env bash
# check_effect_lattice_doctrine.sh — effect lattice doctrine immutability gate.
#
# Fourth doctrine gate (alongside classifier / residual / manifest).
# The effect lattice is the closed authority taxonomy at the compiler
# layer; it pairs with the manifest schema (which the lattice's
# bitset is tested against) and the residual primitive (effect_trace
# is residual kind R4).
#
# Two artifacts must agree:
#   docs/language/effect_lattice.toml — machine-readable manifest
#   compiler/effects.phos             — per-bit bitset layout + masks
#
# This gate fails closed if:
#   - any of the 4 profile alphabets (boot/host/trusted/runtime) is
#     missing from the TOML
#   - the per-profile label list drifts (renamed, added, removed)
#   - the diagnostic-code namespace E-001..E-007 drifts
#   - the .phos source's per-bit comment table does not list every
#     label declared in the TOML alphabets
#   - the profile alphabet bitmasks (31, 2016, runtime mask) do not
#     match the per-bit positions
#   - log-analyzer vocabulary leaks into the source
#
# Apex framing this gate protects: declared authority is a closed,
# machine-readable lattice — not a free-form set of strings.
#
# Exit 0: doctrine intact.
# Exit 1: doctrine violation (named).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

toml="docs/language/effect_lattice.toml"
src="compiler/effects.phos"

if [ ! -r "$toml" ]; then
    echo "[effect-lattice] FAIL: TOML manifest not found at $toml" >&2
    exit 1
fi
if [ ! -r "$src" ]; then
    echo "[effect-lattice] FAIL: source not found at $src" >&2
    exit 1
fi

fails=0
note_fail() {
    fails=$((fails + 1))
    echo "[effect-lattice] FAIL: $*" >&2
}

# -------------------------------------------------------------------
# 1. The 4 profile alphabets must each have an [alphabet.PROFILE]
#    section header in the TOML. Adding a 5th profile is a doctrine
#    change.
# -------------------------------------------------------------------
required_alphabets=(boot host trusted runtime)

for prof in "${required_alphabets[@]}"; do
    if ! grep -q -E "^\[alphabet\.$prof\]" "$toml"; then
        note_fail "missing [alphabet.$prof] section in $toml"
    fi
done

# -------------------------------------------------------------------
# 2. Per-alphabet label sets. Each label must appear in the
#    profile's `labels = [...]` line. The trusted alphabet is
#    explicitly empty (`labels = []`) and must remain empty.
# -------------------------------------------------------------------
boot_labels=(draw ipc mmio sched time)
host_labels=(host-fs-read host-fs-write host-stdout host-stderr host-time-mono host-hash)
runtime_labels=(draw ipc sched time cap-issue cap-revoke)

# Read each alphabet's labels= line. The TOML places the labels
# declaration on the line immediately after the section header, so
# `grep -F` for the literal section header plus -A 2 captures both
# the header and the labels line in a deterministic way.
read_alphabet_line() {
    local prof="$1"
    # Extract the full [alphabet.PROF] section by selecting the range
    # from the section header to (but not including) the next section
    # header, then pull the `labels =` line. The trusted section has
    # comments between the header and `labels = []`, so a fixed -A
    # window doesn't work; sed range bounding is precise.
    sed -n "/^\[alphabet\.$prof\]/,/^\[/{ /^labels[[:space:]]*=/p }" "$toml" | tail -n 1 || true
}

boot_line="$(read_alphabet_line boot)"
host_line="$(read_alphabet_line host)"
trusted_line="$(read_alphabet_line trusted)"
runtime_line="$(read_alphabet_line runtime)"

for label in "${boot_labels[@]}"; do
    if ! echo "$boot_line" | grep -q -E "\"$label\""; then
        note_fail "boot alphabet missing label '$label' in $toml"
    fi
done
for label in "${host_labels[@]}"; do
    if ! echo "$host_line" | grep -q -E "\"$label\""; then
        note_fail "host alphabet missing label '$label' in $toml"
    fi
done
for label in "${runtime_labels[@]}"; do
    if ! echo "$runtime_line" | grep -q -E "\"$label\""; then
        note_fail "runtime alphabet missing label '$label' in $toml"
    fi
done
if ! echo "$trusted_line" | grep -q -E "labels[[:space:]]*=[[:space:]]*\[\]"; then
    note_fail "trusted alphabet must be exactly 'labels = []' (empty); doctrine: trusted uses primitives, not labels"
fi

# -------------------------------------------------------------------
# 3. Diagnostic-code namespace E-001..E-007. Each code must be
#    declared in the [diagnostics] section in the TOML AND mentioned
#    in the .phos source's diagnostic comment block.
# -------------------------------------------------------------------
e_codes=(E-001 E-002 E-003 E-004 E-005 E-006 E-007)

for code in "${e_codes[@]}"; do
    if ! grep -q -E "^$code\b" "$toml"; then
        note_fail "diagnostic code '$code' missing from [diagnostics] in $toml"
    fi
    if ! grep -q -E "$code\b" "$src"; then
        note_fail "diagnostic code '$code' missing from comment block in $src"
    fi
done

# -------------------------------------------------------------------
# 4. The .phos source's per-bit comment table must list every label
#    declared in the TOML alphabets. Source comment format:
#    `//   bit N: profile.label`. We scan for each label as a
#    substring of the source comment block.
#
# Note: runtime shares bits 0,1,3,4 with boot (draw, ipc, sched,
# time) by design — the source comment table only spells out the
# label once (under boot), so the gate accepts a runtime label as
# satisfied if it appears anywhere in the bit-layout comment, not
# necessarily under a `runtime.` prefix.
# -------------------------------------------------------------------
all_labels=()
all_labels+=("${boot_labels[@]}")
all_labels+=("${host_labels[@]}")
# runtime labels that are NOT shared with boot
all_labels+=(cap-issue cap-revoke)

for label in "${all_labels[@]}"; do
    if ! grep -q -E "$label" "$src"; then
        note_fail "bit-layout comment in $src does not mention label '$label'"
    fi
done

# -------------------------------------------------------------------
# 5. Profile alphabet masks. The .phos source declares:
#     boot_alphabet_mask    = 31     (bits 0..4)
#     host_alphabet_mask    = 2016   (bits 5..10)
#     runtime_alphabet_mask = 6155   (bits 0,1,3,4,11,12 = 1+2+8+16+2048+4096)
#
# Drift in any mask number means the bit-layout doctrine has been
# corrupted. The exact decimal numerals are doctrine because they
# encode the precise bit positions for each profile.
# -------------------------------------------------------------------
#
# Mask checks: each `fn <profile>_alphabet_mask` returns one specific
# `EffectBitset { bits: <value> }` literal. The function body is on
# the line(s) following the fn signature; rather than match across
# lines, capture the body via `grep -A` and then assert the literal.
check_mask() {
    local fn_name="$1"
    local expected="$2"
    local body
    body="$(grep -F -A 4 "fn $fn_name() -> EffectBitset {" "$src")"
    if ! echo "$body" | grep -q -F "$expected"; then
        note_fail "$fn_name must contain literal '$expected' in $src"
    fi
}
check_mask boot_alphabet_mask    "bits: 31 "
check_mask host_alphabet_mask    "bits: 2016 "
# Runtime mask is written as a sum: 1+2+8+16+2048+4096.
check_mask runtime_alphabet_mask "bits: 1 + 2 + 8 + 16 + 2048 + 4096"

# -------------------------------------------------------------------
# 6. No log-analyzer vocabulary. Same boundary discipline as residual
#    + manifest gates (non-alphanumeric on both sides catches
#    compound names like severity_level, anomaly_score, etc.).
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
    for f in "$toml" "$src"; do
        while IFS= read -r line; do
            [ -z "$line" ] && continue
            note_fail "forbidden term '$term' in $f: $line"
        done < <(grep -n -i -E "(^|[^A-Za-z0-9])$term([^A-Za-z0-9]|$)" "$f" 2>/dev/null | grep -v -E "FORBIDDEN|MUST NOT|never (use|emit)|anti-pattern" || true)
    done
done

# -------------------------------------------------------------------
# 7. Apex framing. The TOML manifest_version must be the canonical
#    string and the closure formula must reference the subset rule.
# -------------------------------------------------------------------
if ! grep -q -E '^manifest_version[[:space:]]*=[[:space:]]*"phosphoric-effect-lattice-v1"' "$toml"; then
    note_fail "TOML manifest_version must be 'phosphoric-effect-lattice-v1'"
fi
if ! grep -q -E "callee_subset|subset of declared_effects" "$toml"; then
    note_fail "TOML must encode the closure subset rule (declared_effects(g) ⊆ declared_effects(f))"
fi

# -------------------------------------------------------------------
# Summary.
# -------------------------------------------------------------------
echo "============================================================"
echo "  Phosphoric effect lattice doctrine gate"
echo "  TOML:   $toml"
echo "  source: $src"
echo "  framing: 'declared authority is a closed lattice, not free-form'"
echo "============================================================"

if [ "$fails" -eq 0 ]; then
    echo "  [effect-lattice] OK — alphabets, labels, diagnostic codes,"
    echo "                   bit layout, profile masks, vocabulary, and"
    echo "                   apex framing all match doctrine"
    exit 0
fi

echo "  [effect-lattice] $fails violation(s) — see FAIL lines above"
exit 1
