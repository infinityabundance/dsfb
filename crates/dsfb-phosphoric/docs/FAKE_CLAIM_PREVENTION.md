# Fake Claim Prevention

## Real Today

- The archived compiler had a real frontend with lexer, parser, HIR lowering, type checking, effect checking, stable diagnostics, source spans, and a UI conformance corpus before archival.
- The frozen v0 language boundary is documented in `docs/language/V0_FREEZE.md`.
- Unsupported syntax rejection is archive-only material for the general compiler; the active boot-profile emitter also rejects syntax outside its narrow reviewed profile.
- The archive-only compiler enforced conservative move invalidation, affine capability reuse checks, finite-domain match exhaustiveness, and same-module-only call targets.
- The repo has a real bootable UEFI demo that routes one synthetic event, emits a bounded redraw result, and exits deterministically under QEMU.
- The golden booted demo path executes generated BootAsm semantics only; the enforced manifest records no non-Phosphoric runtime objects, `c_objects=none`, `archive_executed=false`, `clang_used=false`, `lld_used=false`, and `external_linker_used=false`.
- Active `.phos` boot-profile sources emit reviewed IR/ASM evidence artifacts before direct EFI image writing.
- The repo has one-command local verification through `make verify`.

## Demo Only

- The bootable artifact is still a narrow generated BootAsm demo path.
- The current GUI path proves a vertical slice, not a full kernel/runtime implementation.
- The button redraw path and event routing are real for the demo, not proof of a full window/input subsystem.
- The archived typed `Ember` scaffold is reference material, not active golden boot code.
- The booted image is not yet a broad Phosphoric OS; the active emitter is a narrow bootstrap profile, not a general compiler.

## Future Work

- Full Phosphoric-generated system image construction.
- Replacement of archive-only code with active Phosphoric source.
- Self-hosting compiler work after lower-level bootstrap parity is proven.
- Broader Phosphoric-to-runtime code generation beyond the current demo-policy slice.
- Any future decision about borrow syntax.
- Any future module/import system.
- Any future isolation story beyond the current single-address-space prototype.
