# Attack Surface Review

This document names the strongest credible criticisms against the current project state.

## Highest-Value Criticisms

- The architecture is ahead of the runtime implementation. Most kernel and OS guarantees are still specified rather than executed.
- The bootable artifact is real, but it is still a narrow generated BootAsm demo path, not a broad PhosphorOS runtime.
- The archived compiler frontend has meaningful archive-only material, and the active tree has one narrow boot-profile emitter, but there is still no general active compiler backend proof and no formal assurance.
- The trust boundary is documented well, but the current system still trusts the active shell emitter, direct PE/COFF image writer/parser, shell/coreutils behavior, `QEMU`, `OVMF`, and firmware behavior.
- The system is still single-address-space and must not claim hostile-task isolation.
- The move checker is conservative, not path-sensitive. That is safer than under-enforcement, but it is not the same thing as complete ownership reasoning.
- Borrow syntax is frozen out rather than implemented. That is honest, but it means the language is narrower than some older docs originally implied.
- Fixed capacities are present in the active boot profile and archived runtime archive-only material, but the live runtime coverage is still narrow compared with the full OS design.
- `Ember` is smaller than a hobby-kernel convenience bucket, but the active BootAsm path still bypasses a real active Phosphoric kernel/runtime.

## What Critics Would Be Right About

- They would be right to call the current executable path a demo, not a finished OS.
- They would be right to ask for a clearer separation between irreducible trusted code and temporary demo policy.
- They would be right to demand evidence for every claim instead of accepting architecture documents alone.
- They would be right to reject any attempt to market the project as more secure than general-purpose systems languages in general.

## What Critics Would Be Wrong About

- They would be wrong to call the project pure architecture cosplay. The repo has archived compiler evidence, active boot-profile source lowering, a bootable UEFI artifact, and a reproducible verification gate.
- They would be wrong to say fixed capacities are an accidental limitation. The docs, active boot profile, archived compiler/runtime evidence, and review rules treat them as a deliberate auditability constraint.
- They would be wrong to say the trust boundary is hidden. The repo now has `STATUS.md`, `CLAIMS.md`, `docs/invariant_trace.md`, `ember/docs/EMBER_TRUST_AUDIT.md`, and `ember/docs/EMBER_MINIMALITY.md` specifically to keep it explicit.

## Review Discipline Required

- Do not let the existence of the demo be used as proof of a real kernel runtime.
- Do not let the existence of language docs be used as proof of compiler enforcement.
- Do not let the existence of conservative ownership checks be overstated as full borrow or path-sensitive reasoning.
- Do not let `Ember` absorb convenience logic that can live above the hardware boundary.
