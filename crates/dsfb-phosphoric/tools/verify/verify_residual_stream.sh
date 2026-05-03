#!/usr/bin/env bash
# verify_residual_stream.sh — residual stream determinism check.
#
# STATUS: STUB. The R1..R7 residual emission infrastructure described
# in docs/FORENSIC_PRIMACY.md is not yet implemented. The producer
# emits no residuals today.
#
# When wired up, this script will:
#   - Replay an incident input through the producer's emitted binary.
#   - Capture the residual stream byte-for-byte.
#   - Compare against a locked reference stream.
#   - Verify chain_hash continuity (chain_hash[n] = H(prev || event_bytes)).
#   - Verify monotonic seq with no gaps.
#   - Run the classifier (phosphoric_drift.phos) and assert the verdict
#     matches the locked DriftClass + structured EXPECTED/ACTUAL fields.
#
# Until the residual emission infra lands, this script exits 0 with an
# informational message so downstream gates (verify-legendary) don't
# fail on its absence.

set -euo pipefail

cat <<'EOF'
[residual_stream] STUB — residual emission infra not yet implemented.
                  See docs/FORENSIC_PRIMACY.md for the ABI contract.
                  This gate will activate when the producer emits R1..R7
                  records at authority boundaries.
EOF

exit 0
