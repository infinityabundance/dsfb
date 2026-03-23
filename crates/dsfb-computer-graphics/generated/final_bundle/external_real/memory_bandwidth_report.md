# Memory Bandwidth Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

| Label | Resolution | bytes read / px | bytes written / px | validation readback / px | estimated memory traffic MB | reads / px | writes / px | readback required in production |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| native_imported | 160x96 | 368 | 12 | 12 | 5.74 | 23 | 3 | false |
| scaled_1080p | 1920x1080 | 368 | 12 | 12 | 775.20 | 23 | 3 | false |
| scaled_4k | 3840x2160 | 368 | 12 | 12 | 3100.78 | 23 | 3 | false |

Readback required in production: `false`.
Readback was used here only for validation, numerical delta checks, and report generation.

## Memory Access / Coherence Analysis

- Buffer access pattern: linear per-pixel reads for current color, reprojected history, motion, depth pairs, and normal pairs, plus three scalar output writes.
- Neighborhood reads: the kernel performs two 3x3 neighborhood traversals over current color, one for local contrast and one for neighborhood-hull gating.
- Coherence expectation: adjacent threads in the 8x8 workgroup read strongly overlapping 3x3 neighborhoods, so the access pattern is locally coherent even though current color is revisited many times.
- Cache-friendliness: the minimum kernel avoids scattered history gathers because reprojection is precomputed before dispatch. That keeps the kernel more cache-friendly than a motion-indirected gather path.
- Cache risk: repeated 3x3 reads from a storage buffer still raise bandwidth pressure on current color, so profiling should confirm whether a texture path or shared-memory staging would reduce traffic materially.
- Optional path impact: any future motion-augmented kernel that reintroduces motion-neighborhood disagreement or in-kernel reprojection will increase cache pressure materially and should be treated as non-minimum.

## What Is Not Proven

- This report is analytical accounting based on the implemented kernel and does not replace external validation with hardware-counter collection.
- Reported traffic is sufficient for reviewer diligence, but not a substitute for per-architecture cache analysis.

## Remaining Blockers

- External validation still needs hardware counters and vendor-specific bandwidth profiling on imported real captures.
