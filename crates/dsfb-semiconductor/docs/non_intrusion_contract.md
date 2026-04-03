# Non-Intrusion Contract

## Formal Definition

DSFB is a deterministic observer. It maps residual streams to structured episodes
without any return path into the process or control space.

Formally:

```
O: R → E
```

Where:

- **R** = ordered sequence of residual observations, upstream alarm snapshots,
  and optional metadata. R is an immutable input surface. DSFB holds no mutable
  reference to R after ingestion; the caller retains full ownership.
- **E** = structured advisory episodes drawn from {Silent, Watch, Review, Escalate},
  annotated with typed labels, a deterministic trace chain, and a policy rationale
  string.

## Explicit Exclusions

The codomain of O is strictly bounded to E. No element of E can change process
state without an explicit human decision outside DSFB.

| What O does NOT map to | Justification |
|---|---|
| Control commands | No write-back API exists in this crate |
| Threshold updates | No parameter-modification API exists |
| Recipe modifications | No process-state write API exists |
| Actuation signals | Actuator path is upstream authority only |
| Controller gains | Gain scheduling is outside DSFB scope |
| SPC control chart parameters | Chart maintenance is upstream authority |

## Rust Enforcement

The observer interface is defined as:

```rust
pub trait DSFBObserver {
    fn ingest(&self, residual: &ResidualSample);
    fn output(&self) -> Vec<PolicyDecision>;
}
```

`ingest` accepts a shared (non-mutable) reference only. The residual value cannot
be modified by DSFB. `output` returns a value type (owned Vec), not a reference
into any upstream data structure. No API in this crate writes to any stream,
threshold, or actuation target.

Test coverage: `tests/no_feedback_test.rs` asserts that no feedback surface token
appears in the serialized output. `tests/deterministic_replay.rs` asserts that
identical inputs produce identical outputs.

## Temporal Independence

DSFB executes on a side-channel tap of the residual stream. The primary
control loop timing is not affected because:

1. DSFB reads a copy of the residual sample, not the original stream object.
2. DSFB output is produced after ingestion completes; it does not block the
   primary control period.
3. If DSFB crashes or is disabled, no upstream system is affected. The
   `fail_safe_isolation_note` field in every advisory output documents this.

## Determinism Guarantee

For identical ordered inputs R₁ = R₂ (element-wise equal, same order):

```
O(R₁) = O(R₂)
```

This is verifiable from saved artifacts. Every run emits `dsfb_traceability.json`
and `dsfb_run_manifest.json`. Any advisory output can be replayed from those
artifacts without re-running the upstream monitoring systems.

## Audit Trail

Every element of E carries:

- `integration_mode = "read_only_side_channel"`
- `chain = "Residual -> Sign -> Motif -> Grammar -> Semantic -> Policy"`
- The specific feature identifiers, timestamps, and run indices that produced it

No step in the chain is skipped. If a step produces no output (e.g., no semantic
match), the entry is omitted from `dsfb_traceability.json` rather than recorded
with a blank field. This preserves the invariant that every persisted trace entry
is fully populated.
