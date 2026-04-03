# Fab Integration Guide

## Exact Placement in the Monitoring Stack

```
Tools → SPC / EWMA / FDC / APC
                ↓
          residual streams
                ↓ (side tap — read-only copy)
          DSFB Observer Layer
                ↓
     structured advisory episodes
     (Silent / Watch / Review / Escalate)
```

DSFB is positioned immediately downstream of the existing monitoring stack. It
consumes the residual stream as a read-only side tap. There is no arrow from
DSFB back to the control path.

**Architecture labels (see `figures/dsfb_fab_integration.svg`):**

- READ-ONLY — DSFB holds immutable references to input samples
- NO FEEDBACK — no output API reaches upstream systems
- NO CONTROL INTERFERENCE — primary control loop timing is unchanged

## Observer Interface

```rust
pub trait DSFBObserver {
    fn ingest(&self, residual: &ResidualSample);
    fn output(&self) -> Vec<PolicyDecision>;
}
```

`ingest` takes a shared reference only. `output` returns an advisory value type.
No method in the trait writes to any upstream system. This is the complete
public surface of the observer.

A full read-only adapter:

```rust
pub trait FabDataSource {
    fn residual_stream(&self) -> Vec<ResidualSample>;
}
```

See `examples/fab_stub.rs` for a minimal adapter that wires a tool residual
buffer to the observer.

## Integration Steps

1. **Identify the residual export point.** This is typically the residual or
   innovation output from the existing SPC/EWMA/APC system — the same scalar
   that triggers existing alarms.
2. **Preserve ordering.** Feed residuals to DSFB in the same timestamp order
   they arrive upstream. DSFB relies on temporal ordering for drift and
   grammar evaluation.
3. **Pass upstream alarm snapshots (optional).** The `UpstreamAlarmSnapshot`
   struct accepts boolean flags for EWMA, SPC, and threshold alarms. DSFB uses
   these as corroboration signals only and does not modify them.
4. **Review DSFB output alongside upstream alarms.** DSFB output is advisory.
   The upstream system remains the authoritative detection mechanism.

## Deployment Modes

| Mode | When to use |
|---|---|
| Per-tool | Residual ownership is tool-local; review is tool-local |
| Per-wafer batch | Review is scheduled around batch release or hold decisions |
| Centralized node | Fab already aggregates alarm review centrally |

All modes O(n) in residual count. No GPU. No training cluster.

## Non-Intrusion Guarantees

- If DSFB is disabled, upstream plant behavior is unchanged.
- If DSFB produces a wrong advisory, the upstream alarms remain unaffected.
- DSFB output is deterministic: identical inputs produce identical outputs.

Full formal statement: see `docs/non_intrusion_contract.md`.
