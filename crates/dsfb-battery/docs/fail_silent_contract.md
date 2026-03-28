# Fail-Silent Contract

This note is crate-local only. It does not edit the paper. It records what
`dsfb-battery` means by invalid-stream handling and fail-silent behavior in the
current implementation.

## Scope

The current production mono-cell path is still a capacity-only, advisory audit
trace over the NASA PCoE B0005-style workflow. The figure pipeline is unchanged.

The fail-silent contract implemented here applies to the DSFB classification
path and audit artifact generation for intervals where the input stream cannot
support a valid residual/drift/slew interpretation.

## What Invalid-Stream Means Here

For the current crate, an interval is treated as invalid when:

- the upstream capacity sample is non-finite, or
- the residual/drift/slew terms derived from that sample are non-finite under
  the current fixed windows

This means a single invalid sample can suppress more than one cycle if the
windowed drift/slew terms remain non-finite until the declared windows refill
with valid data again.

## What Fail-Silent Means Here

During an invalid interval:

- no normal DSFB classification event is emitted
- no normal state-transition event is emitted
- DSFB classification is explicitly suppressed for that interval
- an `invalid_stream_gap` audit record is emitted instead

When the derived residual/drift/slew terms become finite again, normal
classification emission can resume.

## Interface Contract Fields

- `fail_silent_on_invalid_stream`
  Meaning: the emitted contract declares fail-silent behavior for invalid input
  intervals.
- `fail_silent_defined`
  Meaning: fail-silent is a defined design contract of the emitted interface.
- `fail_silent_enforced`
  Meaning: the current implementation actually suppresses classification output
  during invalid intervals instead of only describing that behavior.

## Audit Representation

Invalid intervals are represented with `event_type = "invalid_stream_gap"`.

The event carries:

- `reason_code = "InvalidStreamSuppression"`
- `cycle_index_start`
- `cycle_index_end`
- `audit_fields.stream_valid = false`
- `audit_fields.suppressed_due_to_invalid_stream = true`

The cryptographic hashes in the audit fields still refer to the same run
configuration and input series; they provide artifact integrity, not correctness
proofs.

## Initialization Limit

The healthy baseline window still has to be finite. If the initial healthy
window cannot define a valid envelope, the pipeline returns an explicit error
instead of fabricating a classification.
