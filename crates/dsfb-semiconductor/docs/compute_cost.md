# Compute Cost

DSFB operates with bounded, audit-friendly compute requirements.

## Cost Model

- O(n) over the residual stream
- per-feature linear cost
- no model training
- no GPU required

The current crate writes full audit artifacts, so retained output volume is also linear in the number of residual samples kept for inspection.

## Deployment Modes

### 1. Per-tool

- Run beside one tool or chamber controller.
- Best when residual ownership and operator review are already tool-local.
- Cost scales linearly with that tool's residual volume.

### 2. Per-wafer batch

- Run after each batch closeout over the batch residual archive.
- Useful when review is scheduled around batch release or hold decisions.
- Cost remains linear in the batch residual count.

### 3. Centralized monitoring node

- Aggregate residual taps from multiple upstream systems into one observer node.
- Useful when the fab already centralizes alarm review.
- Total cost is the sum of the incoming residual streams; no training cluster or accelerator is required.
