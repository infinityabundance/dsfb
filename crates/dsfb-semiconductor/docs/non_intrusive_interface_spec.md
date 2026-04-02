# Non-Intrusive DSFB Interface Specification

DSFB is a deterministic, non-intrusive, read-only interpretation layer. It does not replace SPC, EWMA, threshold logic, APC, or controller actuation. Its role is to read upstream residuals and alarms, transform them through a fixed structural stack, and emit advisory interpretations only.

## Contract

- Integration mode: `read_only_side_channel`
- Fixed layer order: `Residual -> Sign -> Syntax -> Grammar -> Semantics -> Policy`
- Inputs are immutable residual observations, upstream alarm snapshots, and optional metadata.
- Outputs are advisory interpretations only: `Silent`, `Watch`, `Review`, or `Escalate`.
- No DSFB API writes back into thresholds, controller gains, recipe parameters, or actuation paths.
- Primary control timing is unchanged because DSFB consumes a side tap of residual/alarm streams.
- Replay is deterministic: identical ordered inputs must yield identical outputs.
- Failure is isolated: if DSFB crashes or is disabled, upstream plant behavior is unchanged.

## Input Surface

`DsfbObserverInput` contains:

- `run_index`
- `timestamp`
- `residuals`
- `upstream_alarms`
- `metadata_pairs`

## Output Surface

`DsfbAdvisoryOutput` contains:

- `run_index`
- `timestamp`
- `advisory_state`
- `layer_order`
- `advisory_labels`
- `advisory_note`
- `fail_safe_isolation_note`

## Explicit Non-Claims

- No control command output exists.
- No threshold-tuning API exists.
- No recipe-write API exists.
- No claim of controller replacement is made.
- No claim of latency benefit is made; the contract is only that DSFB must not add latency to the upstream control loop.
