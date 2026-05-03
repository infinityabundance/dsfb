# Phosphoric Conformance Corpus

> **FREEZE NOTE (Phase 7 Tranche A, endoduction-substrate plan).**
> Non-M-### corpus expansion is frozen. New cases land only when they
> exercise a manifest authority predicate (M-001..M-012), a residual
> classification path, or a forensic-boundary gate. Generic "more
> coverage of grammar productions" cases are not added; they dilute the
> evidence-per-LOC ratio without lighting up any of the twelve outcomes.

The conformance corpus is a grammar-driven, mechanically-enumerated test set that pins what v0 Phosphoric is and is not. Every grammar production has at least one positive test (compiles cleanly) and at least one negative test (rejected with a stable diagnostic code).

This directory is the deliverable of elevation item E2. The runner is `phosphoric-conform.phos` (host program; elevation item E2 deliverable). The runner reads [docs/language/grammar.md](../../docs/language/grammar.md), enumerates productions, and asserts each has a `pass/<production>.phos` and `fail/<production>.phos` here. Missing coverage fails CI.

## Layout

```
tests/conformance/
├── README.md                    (this file)
├── boot/
│   ├── pass/                    one .phos per grammar production
│   └── fail/                    one .phos per non-goal / illegal construct
├── host/                        host-profile conformance
│   ├── pass/
│   └── fail/
├── trusted/                     trusted-profile conformance
│   ├── pass/
│   └── fail/
└── runtime/                     runtime-profile conformance
    ├── pass/
    └── fail/
```

## Required Coverage (from grammar.md)

### Boot profile — pass cases

Each of the following produces at least one `pass/<name>.phos`:

- `module_declaration_simple` — single-segment module path
- `module_declaration_dotted` — multi-segment dotted path
- `profile_clause_present` — explicit `profile boot;`
- `profile_clause_absent` — defaults to boot
- `capability_declaration` — `capability Foo;`
- `struct_declaration_empty`, `struct_declaration_one_field`, `struct_declaration_many_fields`, `struct_declaration_trailing_comma`
- `enum_declaration_unit_variants`, `enum_declaration_tuple_variants`, `enum_declaration_mixed`
- `function_no_params_no_return_no_effects`
- `function_with_params`, `function_with_return`, `function_with_effects`
- `function_full` — params + return + effects
- One pass per type form: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `bool`, `[T; N]`, `Slice[T, N]`, `Result[T, E]`, `Option[T]`, dotted `PathType`
- One pass per statement form: `let`, `let mut`, assignment, expression statement, `for` loop with `#[bound]`, `return` with and without expression
- One pass per expression form: integer literal, `true`, `false`, path expression, tuple-like expression, array literal, parenthesized, `if` with and without `else`, `match` with one arm, `match` with many arms, all binary operators
- One pass per pattern form: `_`, integer pattern, `true`/`false`, ident, variant unit, variant tuple
- One pass per effect label: `draw`, `ipc`, `mmio`, `sched`, `time`

### Boot profile — fail cases

Each of the following produces at least one `fail/<name>.phos` and asserts a stable diagnostic code:

- `trait_declaration` — rejected with `P-???` (parser; trait keyword not recognized)
- `impl_block` — rejected
- `macro_declaration`
- `async_function`
- `recursion_self_call` (kernel-profile subset only)
- `heap_string_literal`
- `unsafe_block`
- `borrow_operator`
- `mutable_borrow_operator`
- `pointer_type`
- `generic_function`
- `closures`
- `while_loop`
- `unrestricted_loop`
- `for_loop_without_bound_annotation` — `B-001`
- `bound_annotation_unprovable` — `B-002`
- `for_loop_negative_bound` — `B-004`
- `external_call_syntax`
- `import_syntax`
- `method_receiver`
- `pattern_guard`
- `lifetime_annotation`
- `floating_point_literal`
- `floating_point_type`
- `string_literal`
- `non_ascii_byte_in_source` — `L-005`
- `unterminated_block_comment`
- `effect_label_unknown` — `E-001`
- `effect_declared_but_unused` — `E-003`
- `cross_profile_effect_mix` — `E-006`
- `same_module_recursion` — typeck cycle rejection
- `move_after_move` — typeck affine violation (`K-007`)
- `capability_duplication` — typeck capability check (`K-009`)
- `non_exhaustive_match` — typeck exhaustiveness (`K-006`)
- `mismatched_arm_types` / `match_arms_divergent` — typeck arm body type uniformity (`K-016`)
- `binary_operand_type_mismatch` — typeck binary operand type compatibility (`K-014`)
- `call_arity_mismatch` — typeck call argument count (`K-002`)
- `field_access_on_non_struct` — typeck base-must-be-struct (`K-003`)
- `variant_access_on_non_enum` — typeck base-must-be-enum (`K-004`)
- `return_type_mismatch` — typeck function-body / declared-return match (`K-012`)
- `assign_unit_to_typed` — typeck assignment rhs type (`K-001`)
- `let_binding_unit_to_typed` — typeck let-binding rhs type (`K-013`)
- `assign_to_immutable`
- `let_without_type_when_inference_fails`

### Host profile — coverage

Required cases listed in [docs/language/host_profile.toml](../../docs/language/host_profile.toml) under `[conformance.required_cases]`:

- pass: `host_fs_read_file`, `host_fs_write_file`, `host_stdout_message`, `host_stderr_message`, `host_time_mono_read`, `host_hash_buffer`, `host_argv_iteration`, `host_capacity_minimum_declaration`
- fail: `host_undeclared_effect`, `host_effect_in_boot_profile`, `host_boot_effect_in_host_profile`, `host_capacity_exceeds_maximum`, `host_disallowed_syscall_attempt`, `host_recursive_walk_attempt`, `host_argv_overflow`, `host_cross_profile_import`

### Trusted profile — coverage

- pass: `trusted_block_cpu_op_cli`, `trusted_block_mmio_read32`, `trusted_block_mmio_write64`, `trusted_block_with_audit_comment`
- fail: `trusted_outside_allowlist` (`T-001`), `trusted_block_non_primitive` (`T-002`), `trusted_block_runtime_selector` (`T-003`), `trusted_block_unknown_region` (`T-004`), `trusted_block_disallowed_msr` (`T-005`), `trusted_block_no_audit_comment` (`T-006`), `trusted_block_unaudited` (`T-007`), `multiple_trusted_blocks_in_function` (`T-008`)

### Runtime profile — coverage

Required cases listed in [docs/language/runtime_profile.toml](../../docs/language/runtime_profile.toml) under `[conformance.required_cases]`.

## Runner Contract

`phosphoric-conform.phos` walks the directory and per-test:

- For `pass/<name>.phos`: invokes `pcc` and asserts exit code 0 with no diagnostic output.
- For `fail/<name>.phos`: invokes `pcc` and asserts exit code non-zero with the expected diagnostic code on stderr. The expected code is encoded in a header comment: `// fail-expected: P-001`.

Missing coverage (a grammar production with no positive test, or a non-goal with no negative test) is a hard CI failure — the runner enumerates required coverage from the manifest TOMLs and the grammar doc.

## Reachable Diagnostic Codes

The K-### codes reachable from the type-checker's top-level entry today are:

- `K-001` (type mismatch) — Assign rhs Unit to typed place
- `K-002` (arity mismatch) — Call with wrong arg count
- `K-003` (field access on non-struct) — Field with literal base
- `K-004` (variant access on non-enum) — variant path on non-enum base
- `K-006` (non-exhaustive match)
- `K-007` (move-after-move) — capability tracker
- `K-008` (use-after-move) — capability tracker
- `K-009` (capability duplication) — same expr id in two arg slots of one Call
- `K-010` (unknown name) — out-of-range arena id
- `K-011` (unsupported type form) — bound 0 on `for`
- `K-012` (return type mismatch) — declared non-unit, trailing block expr is Unit
- `K-013` (let-binding type mismatch) — Unit rhs into typed binding
- `K-014` (binary operator operand types incompatible) — Bool/Int/Unit-mismatched operands
- `K-016` (match arms with divergent value types) — literal-class divergence
- `K-019` (function body produces no value) — no trailing expression on a non-unit function

Reserved-only codes pending future precision work: K-005 (match arm pattern type mismatch), K-015 (if-condition not bool — possibly unreachable in v0), K-017/K-018/K-020 (E16 grammar extension precision codes).

## Status

The required-coverage list above is the contract; individual `.phos` files land incrementally. The conformance gate is enforced from day one of `phosphoric-conform.phos` landing: the gate fails on missing coverage even before all pass/fail cases exist, because the gate itself reads this README's required-coverage list as the source of truth.
