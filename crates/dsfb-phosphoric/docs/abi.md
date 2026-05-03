# Phosphoric v0 Prototype ABI

This document defines the calling-convention assumptions for the initial Phosphoric prototype target.

The ABI is intentionally narrow and target-specific. It exists to make the compiler, `Ember`, and early kernel code agree on value layout and call boundaries for the `x86_64` prototype.

## Target Profile

The initial ABI profile is locked to:

- architecture: `x86_64`
- endianness: little-endian
- boot environment: `UEFI`
- execution target after bring-up: kernel-controlled runtime above `Ember`

Two boundaries matter:

- the firmware boundary, which stays inside `Ember`
- the internal Phosphoric runtime ABI, which higher layers use after bring-up

## Boundary Split

The prototype uses different assumptions at different boundaries:

- `UEFI` entry and firmware service calls use the firmware-required calling convention inside `Ember`
- the internal Phosphoric ABI is defined separately and does not expose firmware calling details to ordinary Phosphoric code

This keeps firmware-specific details out of the language surface.

## Internal Phosphoric ABI

The internal prototype ABI is a simple `x86_64` register-first convention modeled to stay easy to lower from a hosted compiler.

Call rules:

- integer, boolean, enum tag, pointer-free handle, and capability-sized arguments are passed in order through `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`
- additional arguments are passed on the stack
- the stack must be 16-byte aligned at call boundaries
- scalar return values are returned in `rax`
- two-register scalar returns may use `rax` and `rdx`
- larger aggregate returns use caller-provided storage passed explicitly by the frontend or lowering layer

Register preservation:

- caller-saved: `rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8`, `r9`, `r10`, `r11`
- callee-saved: `rbx`, `rbp`, `r12`, `r13`, `r14`, `r15`

## Value Representation Assumptions

The v0 ABI assumes:

- integers use their declared fixed widths
- `bool` is lowered to an integer-sized scalar with only `0` and `1` as valid logical values
- arrays are inline fixed-size aggregates
- structs are fixed-layout aggregates with field order preserved by the ABI profile
- enums are tagged unions with a fixed tag representation chosen by the compiler for the active target
- capabilities are opaque fixed-size values represented as plain machine words or fixed aggregates, never as ambient globals

The exact enum layout algorithm should remain simple and deterministic in v0.

## Function Shape Restrictions

The v0 ABI forbids:

- variadic functions
- stack unwinding across call boundaries
- hidden allocator parameters
- hidden vtable parameters
- implicit exception objects

Effect declarations and capability rules live above the ABI. They do not change how values are passed, only what calls are legal in typed source.

## Error And Result Representation

The ABI expects `Result[T, E]` and `Option[T]` to lower to explicit tagged data.

Rules:

- `Option[T]` lowers to a tag plus payload when niche optimization is not defined by the v0 profile
- `Result[T, E]` lowers to a tag plus explicit payload storage
- no hidden unwinding or exception metadata is used for recoverable failure

The prototype chooses clarity over maximal layout optimization.

## Slice Representation

`Slice[T, N]` is lowered as a bounded non-owning view.

The ABI-level representation includes:

- a base address or base reference to existing storage
- a current logical length
- the compile-time maximum bound `N` enforced by typing and lowering rules rather than requiring runtime-resizable metadata

The slice representation must not imply ownership of heap storage.

## Call-Site Rules

The compiler and runtime must obey:

- arguments are evaluated before the call using explicit move semantics
- moved values are not reused after the call
- effect checking is completed before ABI lowering

Borrow semantics are not part of the current frozen source surface, so this ABI does not currently rely on source-level borrow lowering.

The ABI does not rescue invalid source programs. It assumes earlier compiler stages already rejected them.

## Current Guarantees

This ABI document currently guarantees:

- a single concrete prototype target
- a stable internal call boundary for early compiler work
- explicit, non-unwinding representation of ordinary results
- a clean separation between firmware ABI concerns and the ordinary Phosphoric surface

## Forbidden In v0

The ABI profile forbids:

- multiple competing internal ABIs
- target-dependent implicit allocation behavior
- exception-based failure transport
- hidden metadata that changes ownership semantics
- firmware ABI leakage into ordinary Phosphoric interfaces

## Future Work That Is Not Yet Promised

The following are deferred:

- cross-architecture ABI support
- verified layout proofs
- aggressive enum niche optimization
- FFI ABI commitments beyond the narrow trusted boundary
- externally-visible ABI stability guarantees across major design revisions
