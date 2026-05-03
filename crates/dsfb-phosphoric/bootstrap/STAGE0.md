# Phosphoric Bootstrap — Stage 0

This file documents the **stage 0 bootstrap binary**: the one externally-built artifact in the project's history that the active repo is allowed to depend on, exactly once, exactly externally, exactly hash-pinned.

The active repo contains zero source written in any language other than Phosphoric. The Phosphoric self-hosted compiler `pcc.phos` is the canonical compiler. To build it the first time, a stage 0 binary is required. From stage 1 onward, the chain is self-perpetuating: stage 1 compiles `pcc.phos` to produce stage 2, stage 2 compiles `pcc.phos` to produce stage 3, and the [verify_fixpoint.phos](verify_fixpoint.phos) gate asserts byte-equality from some N onward.

Stage 0 is a one-time-only dependency. The active build does not invoke any external toolchain; it fetches stage 0 by URL and SHA-256, runs it once, and never invokes it again in the same build.

## Active path: phase 0

The doctrine path is **phase 0** — a Phosphoric-source bootstrap built once externally by attesters with hand-coded x86_64 ASM stubs.

- Source: [phase0/phase0_compiler.phos](../phase0/phase0_compiler.phos)
- Subset spec: [phase0/phase0_subset.md](../phase0/phase0_subset.md)
- Attester runbook: [phase0/HANDBOOTSTRAP.md](../phase0/HANDBOOTSTRAP.md)
- Scaffold producer (interim): [phase0/produce_stage0.sh](../phase0/produce_stage0.sh)
- Scaffold binary: `build/phase0/pcc-stage0.bin` (136 bytes, SHA-256 pinned in [bootstrap.toml](bootstrap.toml))

The current bootstrap state is `SCAFFOLD`. See [STAGE0_STATUS.md](STAGE0_STATUS.md) for the full status table.

The historical externally-built stage 0 entry (`stage0-x86_64-linux-2026-04`) is preserved in [bootstrap.toml](bootstrap.toml) with `superseded_by = "phase0-x86_64-linux-2026-04"` for append-only audit trail.

## Why this is the honest answer

A self-hosted language has a chicken-and-egg problem: a Phosphoric compiler must be compiled by something. Three choices:

1. **Externally-pinned bootstrap (this repo's choice).** A trusted binary is produced once from a known commit of a historical pre-Phosphoric implementation, signed, hashed, and made fetchable. The active repo references the binary by hash and never invokes any external toolchain itself. This is the pragmatic baseline.

2. **Bare-metal hand-bootstrap.** A hand-authored x86_64 assembly stub compiles a tiny Phosphoric subset, which compiles `pcc.phos`. Eliminates the external-toolchain dependency entirely. Deferred as long-tail item E15.

3. **Trust the host's compiler.** Rejected: it would mean the active repo depends on whichever toolchain the developer has installed, which is a much larger trust surface than a single hash-pinned binary.

This file describes choice 1 honestly — including the residual trust-in-stage0 assumption, which is not eliminated, only made explicit and small.

## Stage 0 contract

The stage 0 binary is a Linux x86_64 ELF executable that:

- accepts `pcc.phos` source files on its command line
- emits ELF, PE/COFF, or raw boot-IR depending on the requested profile
- produces output that is byte-identical to what stage 1 produces from the same input
- has a known SHA-256 hash recorded in [bootstrap.toml](bootstrap.toml)
- is fetchable from the URL recorded in [bootstrap.toml](bootstrap.toml)
- was produced exactly once, by a tagged commit of the historical pre-Phosphoric implementation, in a recorded build environment

Stage 0 is a single signed binary built once from a snapshot of the historical source; that source has been deleted from the active repo (see [docs/RETIREMENT.md](../docs/RETIREMENT.md)) and is recoverable only from git history at the pinned commit.

## What "trust in stage 0" means and does not mean

The fixpoint test in [verify_fixpoint.phos](verify_fixpoint.phos) proves that *if* stage 1 (built by stage 0 from `pcc.phos`) is honest, then stage 2 and stage 3 are byte-identical to stage 1, which proves stage 1 is a correct compilation of `pcc.phos`.

The test does **not** prove stage 0 itself was honest. A malicious stage 0 could insert a backdoor into stage 1 that propagates through the fixpoint. This is the classic Reflections-on-Trusting-Trust gap.

The defenses are:

- stage 0 is hash-pinned; tampering changes the hash
- stage 0 is community-attested at first build; the build commit, environment, and reviewer signatures are recorded
- multiple independent reviewers are encouraged to rebuild stage 0 from the recovered historical source at the recorded commit and confirm hash equality, in different environments
- the eventual long-tail goal (E15) is to replace stage 0 with a bare-metal stub that has no external-toolchain ancestry

The honest claim is: **stage 0 is a small, signed, community-attested bootstrap. Trust in it is not eliminated; it is made explicit, small, and revocable.** When E15 lands, this file gets a section describing the bare-metal stub and stage 0's role becomes purely historical.

## Build provenance for stage 0

Recorded once, immutable thereafter, in [bootstrap.toml](bootstrap.toml):

- the source commit SHA the binary was built from
- the historical compiler identifier and version used at build time (recorded opaquely; the active repo does not name the toolchain)
- the build host kernel version and CPU architecture
- the SHA-256 of the resulting binary
- the public URL where the binary is hosted
- the GPG-signed attestation from the original builder
- additional attestations from independent reviewers who reproduced the build

`bootstrap.toml` is editable only to *add* attestations. The original entry is append-only. A new stage 0 (e.g. for a new architecture) gets a new entry; the old entry never changes.

## When stage 0 is invoked

Exactly twice in the project's history per environment:

1. **First-build:** when the developer or CI has no `pcc-stage1.bin` yet, the build script downloads stage 0 (verifying its hash against [bootstrap.toml](bootstrap.toml)), runs it once on `pcc.phos` to produce stage 1, hashes stage 1, and caches it.
2. **Cache-miss:** if the cached stage 1 is missing or its hash mismatches, the first-build path repeats.

A normal active build invokes stage 1, stage 2, stage 3 — never stage 0.

## What stage 0 does *not* do

- It is not part of `make verify-legendary`. `make verify-legendary` runs only Phosphoric-built stages.
- It is not the canonical compiler. `pcc.phos` is the canonical compiler; stage 0 is a one-time bootstrap of it.
- It does not influence diagnostic stability. Diagnostic codes are pinned by `pcc.phos` source and the UI corpus; stage 0 is required to reproduce them only at first build.
- It is not maintained. The historical source may have rotted, drifted, or stopped building. The hash-pinned binary is what's trusted.

## Replacement criteria

A new stage 0 binary entry may be added to [bootstrap.toml](bootstrap.toml) when:

- a new target architecture is supported (e.g. aarch64) and the existing stage 0 cannot cross-compile for it
- a critical bug in the original stage 0 was discovered that affected the produced stage 1, and a corrected stage 0 must rebuild stage 1 from scratch
- the bare-metal stub from E15 lands and replaces stage 0 entirely (in which case existing stage 0 becomes historical, not active)

A new entry is always additive; the old entry is preserved with a `superseded_by` field for audit history.

## Verification gates

- [bootstrap.toml](bootstrap.toml) is parsed by `verify_bootstrap_manifest.phos` (host program). The parser asserts every required field is present, every hash is well-formed, every URL is HTTPS and bears a stable host.
- `fetch_and_hash_stage0.phos` (host program; uses `host-fs-write` and an explicit external-fetch helper invoked outside the host profile, then re-enters host profile to hash) downloads the binary and verifies SHA-256 equality.
- `verify_fixpoint.phos` (host program) runs stage 0 → stage 1 → stage 2 → stage 3 and asserts `stage 2 == stage 3` byte-for-byte. Stage 1 is allowed to differ from stage 2 if and only if stage 0 was used; subsequent stages must converge.

## Pointers

- Production status (single source of truth): [STAGE0_STATUS.md](STAGE0_STATUS.md)
- Bootstrap manifest: [bootstrap.toml](bootstrap.toml)
- Build runbook for community attesters: [STAGE0_BUILD.md](STAGE0_BUILD.md)
- Fixpoint gate: [verify_fixpoint.phos](../tools/phosphoric-host/verify_fixpoint.phos)
- Hash-verified fetch: [fetch_and_hash_stage0.phos](../tools/phosphoric-host/fetch_and_hash_stage0.phos)
- Manifest validator: [verify_bootstrap_manifest.phos](../tools/phosphoric-host/verify_bootstrap_manifest.phos)
- Archive: pre-Phosphoric source deleted 2026-04-27 per [docs/RETIREMENT.md](../docs/RETIREMENT.md); recoverable from git history at the commit pinned in [bootstrap.toml](bootstrap.toml) `[stage0.source_provenance].git_commit`
