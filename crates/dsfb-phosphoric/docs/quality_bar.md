# Quality Bar

This document defines the minimum bar for future changes to Phosphoric, Ember, PhosphorOS, and the compiler.

The purpose is to preserve assurance density. A change is not high quality because it is clever; it is high quality when its claim, enforcement point, and test evidence are visible.

## Universal Gate

Before a change is accepted:

- run `make verify`
- keep generated `build/` and `target/` artifacts out of commits
- keep tracked archive or packaging artifacts out of the reviewable source state
- update docs before implementation when the change alters a guarantee
- update `STATUS.md` and `CLAIMS.md` when the repo's actual guarantees change
- update `docs/invariant_manifest.toml` when an enforced invariant changes
- update `docs/BOOTSTRAP_TCB.md` when the booted path or bootstrap trust points change
- keep archived non-Phosphoric bootstrap code inert unless a future plan explicitly reopens that boundary
- keep `README.md`, `STATUS.md`, `CLAIMS.md`, `docs/non_goals.md`, and `docs/BOOTSTRAP_TCB.md` aligned on whether a property is current, a v0 non-goal, or a post-v0 strategic goal
- avoid stronger security claims than the threat model supports
- keep trusted interfaces small and directly reviewable
- no backend milestone may weaken the current runtime-truth claims or remove an existing verification test
- bootstrap-path changes must keep the required QEMU log markers honest and reviewable

## Language And Compiler Gate

Any new language feature or compiler check must include:

- grammar or language-spec documentation when syntax or semantics change
- `docs/language/V0_FREEZE.md` update if the frozen boundary changes
- a stable diagnostic code for any new compiler rejection path
- a source span for any rejection whose failure site exists in source text
- at least one direct compiler test that checks the diagnostic code and span
- at least one UI pass case under `compiler/phosphoric-compiler/tests/ui/pass/`
- at least one UI fail case under `compiler/phosphoric-compiler/tests/ui/fail/` when rejection behavior is part of the feature
- at least one `// expect:` diagnostic assertion in each new fail corpus case
- an entry or update in `docs/invariant_trace.md`
- an entry or update in `docs/invariant_manifest.toml` for each enforced invariant
- no hidden allocation, hidden dynamic dispatch, or ambient authority
- if a checked entrypoint's computed worst-case stack depth increases by more than 10%, update `CLAIMS.md` and the assurance report fixture in the same change
- if an active assurance emitter is reintroduced and its output changes, update the golden report fixtures in the same change
- any future full memory-budget check must be tied to real runtime source objects or an explicitly accepted v0.2 language construct
- any profile-driven backend change must include a stable backend diagnostic code for new rejection paths, a direct backend test, a backend fail-corpus case, and a deterministic generated-artifact fixture

Parser-only support is not enough for a feature that claims semantic enforcement.

Reject the change if it quietly widens the frozen language surface or relies on prose that disagrees with `docs/language/V0_FREEZE.md`.

## Ember Gate

Any new `Ember` operation must state:

- which machine-dangerous category it belongs to
- why it cannot live above `Ember`
- what typed interface is exposed upward, if any
- what invariant would fail if the operation were wrong

Reject the change if it exposes raw MMIO, raw port I/O, page-table mutation, trap manipulation, or privileged CPU state through a convenience API.

## Kernel And GUI Gate

Any new kernel or GUI structure must define:

- fixed capacity
- authority required to access it
- explicit exhaustion behavior
- local failure behavior
- whether it belongs in the runtime TCB

Reject the change if it introduces unbounded runtime growth or global ambient access.

## Evidence Gate

Every important claim must have one of these statuses in `docs/invariant_trace.md`:

- `enforced`
- `archive-only`
- `specified`
- `review-gated`
- `not yet enforced`

Do not leave claims implicit. If active enforcement does not exist yet, mark it honestly as `not yet enforced` or `archive-only`, depending on the evidence.

Every top-level project claim must also be reconcilable with:

- `STATUS.md`
- `CLAIMS.md`
- `docs/FAKE_CLAIM_PREVENTION.md`

## Stop Rule

Stop and update the plan before proceeding if a change requires:

- new heap allocation in a runtime path
- a new trusted boundary
- a new effect label
- a new capability class
- a new boot or hardware dependency
- a stronger isolation or security claim
