# Production Evaluation Checklist

Proven in crate:
- host-realistic supervisory effect
- point vs region ROI separation
- external buffer schema and import path
- GPU-executable minimum kernel

Requires external validation:
- real engine buffer export into the schema
- GPU profiling on imported captures
- fair in-engine comparison against strong heuristics
- non-ROI penalty behavior on production scenes

Status:
- external-capable = `true`
- externally validated = `false`
- actual GPU timing measured = `true`

## What Is Not Proven

- This checklist does not claim production readiness.

## Remaining Blockers

- real engine validation
