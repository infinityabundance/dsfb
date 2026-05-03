#!/usr/bin/env bash
# G1 — verify_no_excess_grammar
#
# Forensic role: nothing was *expressible* outside the v0 grammar.
# Walks every *.phos under the repo and asserts every AST node tag is in
# the frozen v0 tag set. Delegates to build/host-tools/check_grammar_v0_only
# which is itself a Phosphoric host program (its source is in
# tools/phosphoric-host/, builds once Stage 0 lands).
#
# Exit: 0 pass, 1 boundary violation, 2 scaffolding missing (warn-skip).

set -euo pipefail

if [ -x build/host-tools/check_grammar_v0_only ]; then
    build/host-tools/check_grammar_v0_only
else
    echo "[scaffold] tools/phosphoric-host/check_grammar_v0_only.phos not yet built"
    exit 2
fi
