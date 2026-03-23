# Next Step Matrix

| Area | Current status | Next action | Negative outcome to watch |
| --- | --- | --- | --- |
| GPU path | measured | Run `run-gpu-path` on evaluator hardware | Kernel timing too high or numeric mismatch vs CPU |
| External handoff | external-capable=true, externally validated=false | Export one real frame pair into the schema | Imported buffers expose missing assumptions or normalization mismatch |
| Competitive baseline | mixed outcomes surfaced | Re-run strongest heuristic on imported captures | Heuristic wins broadly, collapsing DSFB framing to niche-only use |

## What Is Not Proven

- This matrix does not claim any of the next actions will succeed.

## Remaining Blockers

- external evaluator execution still needs to happen
