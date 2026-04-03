# Fab Data Integration

DSFB is positioned as a read-only observer that consumes residual side taps from existing semiconductor monitoring systems. The integration point is downstream of the fab's current SPC, EWMA, APC, or FDC stack.

## Inputs

- etch chamber traces
- lithography drift signals
- FDC residual streams

## Read-Only Contract

```rust
trait FabDataSource {
    fn residual_stream(&self) -> Vec<ResidualSample>;
}
```

The contract is deliberately narrow.

- DSFB reads residuals that already exist in the monitoring stack.
- DSFB does not write thresholds, recipes, or controller parameters.
- DSFB emits structured episodes, semantic tags, and policy suggestions as observer-only outputs.

## Integration Path

1. Export residuals from the incumbent tool or fab monitoring layer.
2. Preserve upstream timestamps, feature identifiers, and tool metadata.
3. Feed the ordered residual stream into the DSFB observer path.
4. Review DSFB outputs beside the incumbent alarms, not in place of them.

## Tool-Level Inputs

- Etch chamber traces should be reduced to the residual or innovation stream already used by the tool's monitoring stack.
- Lithography drift signals should preserve the tool-side timestamp order so DSFB can retain drift and fragmentation structure.
- FDC residual streams should remain immutable once exported to DSFB so replay stays deterministic.

## Example Adapter

See `examples/fab_stub.rs` for a minimal read-only adapter that exposes tool residuals through `FabDataSource`.
