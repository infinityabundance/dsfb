# Phase 0 — bare-metal Phosphoric bootstrap

Phase 0 is the substrate's escape from any external-toolchain dependency. It is a Phosphoric `.phos` source for a minimal-subset compiler plus an external hand-coded x86_64 assembly stub that runs the `.phos` source once to produce stage 1.

The active repo hosts the `.phos` source and the runbook. The ASM stub is **not** in the active repo — it is hand-authored externally by attesters following [HANDBOOTSTRAP.md](HANDBOOTSTRAP.md), and the resulting binary is hash-pinned in [bootstrap.toml](../bootstrap/bootstrap.toml).

## Doctrine

The active repo's source surface is exclusively `.phos`. No external-language source ever enters the tree. The bootstrap chicken-and-egg is resolved by:

1. Phase 0 `.phos` source pinned in this directory ([phase0_compiler.phos](phase0_compiler.phos)).
2. Phase 0 subset specification pinned in [phase0_subset.md](phase0_subset.md).
3. External attesters hand-author x86_64 ASM that interprets the subset and compiles `phase0_compiler.phos` into `pcc-stage0.bin`.

## What phase 0 needs to compile

Phase 0 must compile [compiler/pcc.phos](../compiler/pcc.phos) and the modules it imports — that is the entire scope. It does not need to be a full v0 compiler. Per [phase0_subset.md](phase0_subset.md), the accepted surface is the strict subset of v0 that pcc.phos uses.

## Why this is honest

Phase 0 does not eliminate the trusted-trust gap; it shrinks it. A malicious attester's hand-authored ASM could insert a backdoor that propagates into stage 1 and survives the fixpoint. Defenses:

- Multiple attesters with different toolchains, kernels, and CPU microarchitectures.
- Hash-pinning of the resulting binary.
- The eventual long-tail goal: a bare-metal stub that runs on an OTP-fused boot ROM with no software-mediated trust at all. That is E15+; this is the bridge.

## Files

| File | Role |
|---|---|
| [phase0_compiler.phos](phase0_compiler.phos) | The minimal-subset compiler. 1389 LOC of Phosphoric source: real lexer, real iterative parser, type-check, acyclicity, ELF emit. Function-body lowering is the remaining piece. |
| [phase0_subset.md](phase0_subset.md) | Pinned spec of the subset phase0_compiler.phos accepts. |
| [HANDBOOTSTRAP.md](HANDBOOTSTRAP.md) | External attester runbook. Not the active build path. |

The producer that emits the scaffold-tier binary lives **out of tree** at `untracked/internaldocs/phase0_producer/produce_stage0.sh` per the doctrine that no non-Phosphoric source enters the active repo.

`make verify` does **not** invoke phase 0. Phase 0 runs once, externally, by attesters.
