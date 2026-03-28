# Mission Bus Mapping

Status: Conceptual engineering mapping only. No deployed MIL-STD-1553, ARINC 429, or ARINC 664 stack is implemented here.

Transport-agnostic DSFB signal dictionary:

- `grammar_state`
- `tri_state_color`
- `reason_code`
- `validity_token.sequence_id`
- `validity_token.stream_valid`
- `lead_time_vs_threshold_cycles`
- `advisory_text`

## MIL-STD-1553

Suggested conceptual fields:

- Word 1: state code, color code, advisory-only flag
- Word 2: reason code, validity flag, sequence-id fragment

## ARINC 429

Suggested conceptual labels:

- Label A: state/color summary
- Label B: reason code + validity flag
- Label C: lead-time indication

## ARINC 664

Suggested conceptual payload:

- advisory summary object
- validity token object
- optional narrative string

These mappings are intended for integration planning and ICD discussion only.
