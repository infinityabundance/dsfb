# Legendary Review

## Why is this not just another hobby OS?

Because the repo is not relying on aspirational architecture alone. It has a constrained threat model, a named TCB, a frozen language boundary, a compiler UI corpus, stable diagnostics, a bootable artifact, and a one-command verification gate. See `STATUS.md`, `CLAIMS.md`, `docs/invariant_trace.md`, `make verify`, and `tools/verify/check_all.sh`.

## Why is this not architecture cosplay?

Because the project has executable evidence tied to its claims:

- compiler enforcement through unit tests, direct diagnostic-span tests, frontend pipeline tests, and UI corpus tests
- a bootable UEFI demo under `QEMU`
- explicit trust-boundary and minimality documents under `ember/docs/`

The runtime is still incomplete, but the repo does not hide that. See `STATUS.md`, `CLAIMS.md`, `docs/FAKE_CLAIM_PREVENTION.md`, and `ember/docs/EMBER_TRUST_AUDIT.md`.

## Why is this not fake language complexity?

Because the frozen v0 language surface is intentionally small and explicitly excludes borrow syntax, FFI, traits, macros, async, recursion beyond the rejected kernel-profile subset, and general ecosystem growth. The project is reducing surface area, not inflating it. See `docs/language/V0_FREEZE.md`, `LANGUAGE_NON_GOALS.md`, and the UI fail corpus.

## Why is this not “general-purpose systems languages but worse”?

Because the project is not trying to replace general-purpose systems languages or out-generalize it. The point is a narrower language for a narrower domain with a smaller trust target and fixed-capacity rules. That narrower scope would be a weakness for general-purpose work and a strength for auditability if the enforcement story keeps improving. See `README.md`, `LANGUAGE_NON_GOALS.md`, and `CLAIMS.md`.

## Why does the trust boundary matter?

Because reviewability depends on knowing where machine-dangerous operations stop. `Ember` is the named boundary for firmware entry, validated hardware handoff, raw debug I/O, the current QEMU exit path, and typed low-level boundary descriptions. That boundary is audited and its reducible policy code is called out explicitly. See `docs/tcb.md`, `ember/docs/safety_boundary.md`, `ember/docs/EMBER_MINIMALITY.md`, and `ember/docs/EMBER_TRUST_AUDIT.md`.

## Why are fixed capacities a strength?

Because they turn unbounded runtime growth into a design error instead of a surprise behavior. Even where the runtime is still specified rather than executed, the docs and review gates treat fixed capacities as part of the security and auditability model, not as an embarrassment. See `docs/language/memory_model.md`, `kernel/docs/task_model.md`, `kernel/docs/ipc.md`, `kernel/docs/window_model.md`, and `docs/invariant_trace.md`.

## Why is this reviewable?

Because the repo now exposes its state in a way a hostile reviewer can check:

- `STATUS.md` says what is enforced, partial, demo-only, or missing
- `CLAIMS.md` ties each claim to proof and missing proof
- `docs/invariant_trace.md` maps invariants to evidence
- `docs/ATTACK_SURFACE_REVIEW.md` names the strongest criticisms directly
- `docs/FAKE_CLAIM_PREVENTION.md` separates real today, demo only, and future work

That is stronger than vague optimism because it gives reviewers something concrete to attack and verify.

## Why is this reproducible?

Because the repo has a root verification entrypoint, lower-level verification scripts, explicit expected demo logs, and a release packaging script that stages the current evidence set from source. The claim is local reproducibility for review, not bit-for-bit determinism across arbitrary hosts. See `make verify`, `tools/verify/check_compiler.sh`, `tools/verify/check_docs.sh`, `tools/qemu-run/run_uefi_demo.sh`, `docs/repro_build.md`, and `docs/DETERMINISTIC_BUILD_VERIFICATION.md`.
