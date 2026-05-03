# Phosphoric Host Tooling

This directory holds the elevation toolchain — every verifier, checker, attestation emitter, and equivalence harness.

**Every program here is Phosphoric host-profile source.** No external-language source of any kind. The host profile is defined in [docs/language/HOST_PROFILE.md](../../docs/language/HOST_PROFILE.md).

## Inventory

| Tool | Item | Role |
| --- | --- | --- |
| [phosphoric_invariant_check.phos](phosphoric_invariant_check.phos) | E1 | Walks invariant manifest, asserts evidence pointers |
| [phosphoric_conform.phos](phosphoric_conform.phos) | E2 | Grammar-driven coverage gate over `tests/conformance/` |
| [phosphoric_test_runner.phos](phosphoric_test_runner.phos) | E4 | Runs ~30 kernel tests against deterministic Ember stub |
| [phosphoric_lower_eq.phos](phosphoric_lower_eq.phos) | E5 | HIR ⇄ boot_ir_v1 ⇄ boot_asm_v1 byte-equivalence |
| [phosphoric_repro_diff.phos](phosphoric_repro_diff.phos) | E7 | Two-run byte diff over `build/` |
| [phosphoric_attest.phos](phosphoric_attest.phos) | E8 | dsfb-gray per-checkpoint attestation (SARIF + in-toto + DSSE) |
| [phosphoric_effect_check.phos](phosphoric_effect_check.phos) | E9 | Lattice property checker over `effect_lattice.toml` |
| [phosphoric_fuzz.phos](phosphoric_fuzz.phos) | E11 | Deterministic-seed grammar fuzzer |
| [phosphoric_bound_check.phos](phosphoric_bound_check.phos) | E14 | Annotated loop-bound verifier |
| [check_writer_audit_lines.phos](check_writer_audit_lines.phos) | E6 | PE writer audit-line cross-reference |
| [check_tcb_budget.phos](check_tcb_budget.phos) | E12 | TCB LOC ceiling enforcement |
| [check_trusted_blocks.phos](check_trusted_blocks.phos) | E0e | `trusted!` block ↔ EMBER_TRUST_AUDIT.md cross-reference |
| [check_host_profile_separation.phos](check_host_profile_separation.phos) | E0a | Cross-profile contamination rejection |
| [check_phosphoric_only.phos](check_phosphoric_only.phos) | E0c | Reject non-Phosphoric source extensions outside `archive/` (policy in [forbidden_extensions.toml](forbidden_extensions.toml)) |
| [verify_fixpoint.phos](verify_fixpoint.phos) | E0c | stage{N+1} = stage{N+2} byte-equality |
| [fetch_and_hash_stage0.phos](fetch_and_hash_stage0.phos) | E0c | Bootstrap binary download + SHA-256 verification |
| [verify_bootstrap_manifest.phos](verify_bootstrap_manifest.phos) | E0c | bootstrap.toml schema validation |
| [check_retirement_dates.phos](check_retirement_dates.phos) | E13 | RETIREMENT.md review-date enforcement |

## Build

Each tool compiles to a single statically-linked Linux x86_64 ELF via the host-profile codegen path in `pcc.phos`. The Makefile target `make host-tools` invokes `pcc` once per tool. Output: `build/host-tools/<name>` per tool.

## Status

As of 2026-04-27 every tool here is **scaffolding-only** — module declaration, profile, key types, entrypoint signature, and a TODO body. Real bodies land per-tool with the corresponding elevation milestone. The tooling is honest about its current state: each tool's `main` returns 0 with a stderr note saying "scaffolding; not yet implemented".

This pattern matches the project's framing-first discipline. The contract (signature, diagnostics, output schema) is committed first; the implementation lands against the committed contract.
