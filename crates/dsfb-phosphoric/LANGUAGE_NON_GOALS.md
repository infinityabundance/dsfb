# Language Non-Goals

This file states what Phosphoric is not trying to become.

It exists to keep the language from drifting into a broad ambition project that the current architecture and enforcement story cannot support.

## Hard Non-Goals

- Phosphoric is not trying to replace general-purpose systems languages.
- Phosphoric is not trying to become a general-purpose systems language.
- Phosphoric is not trying to grow a broad package ecosystem in v0.
- Phosphoric is not trying to make high-abstraction programming fashionable for this project.
- Phosphoric is not trying to maximize ergonomics at the cost of auditability.

## Ultra-Razor Non-Goals (2026-04-27 scope recession)

Per the Ultra-Razor direction (`.claude/plans/have-the-panel-deeply-zippy-wozniak.md` § "Ultra-Razor Direction") and [docs/PHOSPHOROS_DESIGN.md](docs/PHOSPHOROS_DESIGN.md):

- **No application-class architectures** as deployment targets. cortex-a, aarch64 server, x86_64 server, RV64GC application class — all out. The five supported arches (x86_64 dev, cortex_m33, cortex_m0plus, riscv_rv32imc, esp32_c3) are a closed set per [docs/language/hardware_features.toml](docs/language/hardware_features.toml).
- **No virtual memory / MMU** in the kernel/runtime. MPU/PMP/SAU is the protection surface.
- **No vendor SDK linkage.** pico-sdk, esp-idf, ARM CMSIS, vendor HAL libraries are not used. The razor compiles its own bytes.
- **No proof effort that competes with seL4.** Targeted formal elements where they add value (effect lattice, capability prover) are kept; full functional-correctness machine-checked proofs are explicitly *not* a goal.
- **No networking on $5 chips.** Even when the silicon (e.g., ESP32) supports Wi-Fi, the razor does not use it. Networking is v0.2+ if at all.
- **No filesystem / persistent storage** beyond OTP for trust roots.
- **No SMP / preemption** in v0.1. Cooperative-yield baseline; timer-driven preemption is *prepared* but not enabled.
- **No floating-point.** Already a v0 non-goal; reaffirmed for embedded targets where FPU absence is the rule.

## Explicit Rejections

- no “general-purpose systems languages, but safer” marketing
- no compatibility-first surface designed to ease porting mainstream software
- no trait or macro expansion to simulate a rich generic ecosystem
- no heap-backed convenience types introduced for developer comfort
- no broad standard library ambitions in v0
- no language growth driven by aesthetic symmetry instead of enforcement value

## What This Means In Practice

- If a feature widens the review surface without materially strengthening the enforcement story, reject it.
- If a feature mainly exists to make the language feel more mainstream, reject it.
- If a feature is easier to explain as “future work” than to enforce today, keep it out of v0.
- If the language starts to look like a platform for arbitrary application development, the project has drifted.

## Review Rule

Reject a language change if it pushes Phosphoric toward:

- general-purpose positioning
- ecosystem-first design
- abstraction for its own sake
- convenience features that weaken determinism, explicit authority, or auditability
