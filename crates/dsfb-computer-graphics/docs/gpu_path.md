# GPU Path

This crate now includes a hardware-facing timing path, but it is still an honest partial result.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## What Exists Now

- a reproducible timing command: `cargo run --release -- run-timing --output <dir>`
- per-stage timing for:
  - reprojection
  - supervision
  - resolve
- analytical op, read, write, and memory-traffic estimates
- minimum, host-realistic, and research/debug timing entries
- a high-resolution selected-scenario proxy entry

Generated outputs:

- `generated/.../timing_report.md`
- `generated/.../timing_metrics.json`

## What Was Actually Measured

In the current environment, the crate measures CPU execution only.

The timing report therefore declares:

- measurement kind = `cpu_only_proxy`
- actual GPU timing measured = `false`

This is intentional. The crate must not fabricate GPU timings.

## What The Current Timing Path Proves

- the supervisory pass has a concrete computational shape
- stage costs can be separated
- memory traffic and buffer burden can be bounded analytically
- the minimum path is materially smaller than the debug path

## What It Does Not Prove

- real GPU milliseconds
- cache behavior on NVIDIA, AMD, or Intel hardware
- final engine pass scheduling
- shipping-performance claims

## Next Honest Step

The next step for stronger diligence is a real GPU implementation or engine-adjacent compute prototype that measures:

- GPU time per stage
- memory traffic or bandwidth counters
- reduced-resolution trust variants
- optional motion-augmented path cost

Until that exists, the crate reports CPU-only proxy timing and says so explicitly.
