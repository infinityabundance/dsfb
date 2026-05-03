#!/usr/bin/env bash
# check_pcc_fail_corpus_stable.sh — v0.1 step 4 done-criterion gate.
#
# Step 4 collapsed parser P-009/P-013/P-019 emission into a single
# check_v0_forbidden() helper. The lexer was extended to recognise the
# 10 forbidden keyword forms (async/trait/impl/macro/unsafe → P-009;
# while/loop → P-013; import/extern/use → P-019). Marker contract per
# the panel directive: existing fail-expected codes/spans remain
# stable; lexer-side reserved-token evidence maps through the parser
# unsupported-token path so the application-facing code stays P-### where
# the fixture expects P-###.
#
# Until v0.1 step 6 fixpoint lands, pcc.phos cannot run end-to-end;
# this gate is structural — it walks the UI fail corpus and asserts:
#
#   (1) every `// fail-expected: P-NNN` marker references a code that
#       is reserved in the parser code-table comment block in
#       compiler/parser.phos.
#
#   (2) for the v0.1-step-4 collapse codes (P-009 / P-013 / P-019),
#       check_v0_forbidden() in compiler/parser.phos carries the
#       matching p_error(9, …) / p_error(13, …) / p_error(19, …) emit
#       arms for each forbidden keyword variant.
#
#   (3) the lexer's keyword-dispatch tables in compiler/lexer.phos
#       contain the cmp_bytes byte-pattern for each of the 10
#       lexer-tagged forbidden lexemes (async/trait/impl/macro/
#       unsafe/while/loop/import/extern/use).
#
# This gate is NOT a general conformance runner. It does not invoke
# pcc; it does not parse fixture bodies. It locks the wiring in repo
# so that any drift between the lexer's reserved-token list, the
# parser's collapse helper, and the corpus markers is surfaced.
#
# Exit: 0 stable, 1 contract drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

parser="compiler/parser.phos"
lexer="compiler/lexer.phos"

[ -r "$parser" ] || { echo "[fail-corpus-stable] missing $parser" >&2; exit 2; }
[ -r "$lexer"  ] || { echo "[fail-corpus-stable] missing $lexer"  >&2; exit 2; }

echo "============================================================"
echo "  pcc.phos UI fail-corpus stability gate (v0.1 step 4)"
echo "============================================================"

fail=0
note() { echo "[fail-corpus-stable] FAIL: $*" >&2; fail=1; }

# ---------------------------------------------------------------------------
# (3) Lexer-tagged forbidden keyword recognition.
# Each entry: human name + ASCII byte sequence as it appears in
# cmp_bytes(s, i, [...], N). The lexer slim uses cmp_bytes inline; we
# grep for the literal byte pattern.
# ---------------------------------------------------------------------------

declare -A kw_pattern=(
    ["async"]='\[97,115,121,110,99,'
    ["trait"]='\[116,114,97,105,116,'
    ["impl"]='\[105,109,112,108,'
    ["macro"]='\[109,97,99,114,111,'
    ["unsafe"]='\[117,110,115,97,102,101,'
    ["while"]='\[119,104,105,108,101,'
    ["loop"]='\[108,111,111,112,'
    ["import"]='\[105,109,112,111,114,116,'
    ["extern"]='\[101,120,116,101,114,110,'
    ["use"]='\[117,115,101,'
)

for kw in async trait impl macro unsafe while loop import extern use; do
    if ! grep -qE "${kw_pattern[$kw]}" "$lexer"; then
        note "lexer.phos missing forbidden-keyword cmp_bytes pattern for '$kw'"
    fi
done

# ---------------------------------------------------------------------------
# (2) Parser collapse helper carries each forbidden-keyword arm.
# ---------------------------------------------------------------------------

# P-009 family — async, trait, impl, macro, unsafe — each must produce p_error(9, …)
for variant in Async Trait Impl Macro Unsafe; do
    if ! grep -qE "Keyword::${variant}[[:space:]]+=>[[:space:]]+Some\(p_error\(9, t.start, t.end\)\)" "$parser"; then
        note "check_v0_forbidden missing p_error(9, …) arm for Keyword::${variant}"
    fi
done

# P-013 family — while, loop — each must produce p_error(13, …)
for variant in While Loop; do
    if ! grep -qE "Keyword::${variant}[[:space:]]+=>[[:space:]]+Some\(p_error\(13, t.start, t.end\)\)" "$parser"; then
        note "check_v0_forbidden missing p_error(13, …) arm for Keyword::${variant}"
    fi
done

# P-019 keyword family — import, extern, use — each must produce p_error(19, …)
for variant in Import Extern Use; do
    if ! grep -qE "Keyword::${variant}[[:space:]]+=>[[:space:]]+Some\(p_error\(19, t.start, t.end\)\)" "$parser"; then
        note "check_v0_forbidden missing p_error(19, …) arm for Keyword::${variant}"
    fi
done

# parse_item dispatches to check_v0_forbidden before the per-kind keyword match.
if ! grep -qE 'check_v0_forbidden\(t\)' "$parser"; then
    note "parse_item / consume sites must call check_v0_forbidden(t)"
fi

# ---------------------------------------------------------------------------
# (1) Every UI fail corpus marker references a parser-reserved code.
# ---------------------------------------------------------------------------

# Reserved code numbers from the stable code-table comment blocks in
# both compiler/parser.phos (per-pass list) and compiler/diagnostic.phos
# (cross-pass registry). Either source is sufficient evidence that a code
# is registered.
reserved_codes=$(
    {
        grep -oE '^//[[:space:]]+P-[0-9]+' "$parser"
        grep -oE 'P-[0-9]+' compiler/diagnostic.phos
    } | grep -oE 'P-[0-9]+' | sort -u | tr '\n' ' '
)

corpus_files=$(find tests/conformance -type f -name '*.phos' | sort)
marker_count=0
p_marker_count=0

for f in $corpus_files; do
    marker=$(grep -oE '//[[:space:]]*fail-expected:[[:space:]]*[A-Z]-[0-9]+' "$f" | head -1 || true)
    [ -n "$marker" ] || continue
    marker_count=$((marker_count + 1))
    code=$(echo "$marker" | grep -oE '[A-Z]-[0-9]+')
    case "$code" in
        P-*)
            p_marker_count=$((p_marker_count + 1))
            if ! echo " $reserved_codes " | grep -q " $code "; then
                note "$f references $code which is not in compiler/parser.phos reserved code table"
            fi
            ;;
        L-*|K-*|E-*|G-*|S-*|B-*|H-*|T-*|R-*|W-*|C-*|I-*|X-*|M-*)
            : # other prefixes are out of scope for this step-4 gate
            ;;
        *)
            note "$f carries non-reserved-prefix marker $code"
            ;;
    esac
done

[ "$fail" = "0" ] || exit 1

echo "  reserved P-codes  : $reserved_codes"
echo "  fail-corpus files : $marker_count"
echo "  P-marker files    : $p_marker_count"
echo "  lexer reserved kw : async/trait/impl/macro/unsafe + while/loop + import/extern/use (10 patterns)"
echo "  parser collapse   : check_v0_forbidden() emits p_error(9|13|19, …) per Keyword arm"
echo "  parse_item wiring : check_v0_forbidden(t) called at item dispatch"
echo "  [fail-corpus-stable] OK — UI fail-corpus markers stable; lexer/parser collapse wired in repo"
exit 0
