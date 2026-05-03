# Phosphoric Stage 0 — Build & Attestation Procedure (HISTORICAL)

> **NOTICE:** The active doctrine path is now **phase 0**. See [phase0/HANDBOOTSTRAP.md](../phase0/HANDBOOTSTRAP.md) for the active attester runbook. This file is preserved for audit history; it documents the generic procedure that the phase 0 specialization replaces.

This file documents the **operational steps** an attester follows to reproduce the stage 0 binary recorded in [bootstrap.toml](bootstrap.toml).

The narrative for *why* stage 0 exists, what it is, and what trust in it means is in [STAGE0.md](STAGE0.md). This file is the runbook a reviewer reads when they want to verify the binary themselves.

`make verify` does **not** execute these steps. They run once, externally, by a builder; subsequent attesters re-run them in a clean environment to confirm the hash matches. The active repo never invokes any external toolchain.

## Preconditions

A reviewer needs:

1. A clean Linux x86_64 host (a fresh container or VM is preferred to limit environmental drift).
2. The historical compiler identified by `bootstrap.toml`'s `[stage0.build_environment].compiler_id` and pinned to `compiler_version`. The active repo does not name which compiler was used; the historical builder recorded this opaquely so the active project's surface stays clean.
3. A copy of the historical pre-Phosphoric source. Three sources, in order of preference:
   - The `phosphoric-history` repo URL recorded at `bootstrap.toml`'s `[stage0.source_provenance].history_repo_url`, checked out to `git_commit`.
   - The active repo's git history, walked back to the commit referenced by `[stage0.source_provenance].git_commit` (recoverable until that commit is pruned).
   - A previously-saved snapshot, if the reviewer has one.
4. A trusted GPG keyring containing the original builder's public key.
5. Network access to `bootstrap.toml`'s `binary_url` (only required if the reviewer wants to compare their build against the published binary; not required for re-building from source).

A reviewer who reproduces stage 0 *without* network access — building from source, hashing locally, comparing only against the hash text in `bootstrap.toml` — has the strongest possible attestation. The published binary is a convenience, not a trust root.

## Step 1 — Recover the historical source

Pre-Phosphoric source is no longer in the active repo (deleted 2026-04-27 per [docs/RETIREMENT.md](../docs/RETIREMENT.md)). Recover it from one of the three sources above.

Confirm the commit SHA matches `bootstrap.toml`'s `[stage0.source_provenance].git_commit` exactly. If it does not, stop — do not attest a different source.

## Step 2 — Pin the historical toolchain

The historical builder recorded `compiler_id` and `compiler_version` in `bootstrap.toml`'s `[stage0.build_environment]`. Install that exact compiler at that exact version on the clean host. The active repo does not document how — the reviewer follows the historical builder's external instructions.

If `toolchain_pinned = true`, the toolchain itself has a hash-pinned source recorded in a sibling document. Without that pinning, the reproduction is "best-effort" — the same source compiled by a slightly different toolchain may produce a slightly different binary.

## Step 3 — Build with deterministic settings

From the historical source root, build with the deterministic flags recorded in `bootstrap.toml`'s `[reproduction].deterministic_build_flags`. Those flags pin codegen-units, link-time optimization, and symbol-stripping — three known sources of build non-determinism. The exact command-line invocation is part of the historical builder's external runbook; the active repo records the *intent* (single-codegen-unit, full-LTO, symbols-stripped) without naming the toolchain that interprets it.

The output binary is renamed to `pcc-stage0.bin` (canonical basename pinned in `bootstrap.toml`'s `[reproduction].expected_binary_basename`).

## Step 4 — Hash and compare

```
sha256sum pcc-stage0.bin
```

Compare the 64-character hex output against `bootstrap.toml`'s `binary_sha256`. If they match exactly, the reproduction succeeds. Any divergence — even one byte — means either the toolchain differs, the source differs, or one of the build flags differs. Investigate before attesting.

The exact commands are pinned in `bootstrap.toml`'s `[reproduction]` table:
- `sha256_command` — the canonical hash tool (`sha256sum`); attesters using BSD `shasum -a 256` produce the same output but the canonical name is recorded for reproducibility.
- `attestation_command` — the canonical signing invocation (`gpg --detach-sign --armor`).
- `expected_binary_basename` — `pcc-stage0.bin`; renaming the binary is permitted but the canonical basename is what `verify_fixpoint.phos` expects.
- `expected_binary_size_min_bytes` / `max_bytes` — sanity bounds. A 256-byte binary is broken; an 80-MB binary is broken. Inside the range, the hash is the authoritative gate.
- `attestation_filename_pattern` — `<binary_sha256>.sha256.asc`. The detached signature filename embeds the hash so multiple attestations across versions don't collide.

## Step 5 — Attest

After a successful match, the reviewer signs an attestation:

```
sha256sum pcc-stage0.bin > stage0.sha256
gpg --detach-sign --armor stage0.sha256
```

The detached signature is published at a stable HTTPS URL. The reviewer then opens a PR adding an entry to `bootstrap.toml`'s `[[stage0.attestation]]` array:

```toml
[[stage0.attestation]]
attestor      = "<reviewer GPG fingerprint>"
signed_at     = "<ISO-8601 timestamp>"
signature_url = "<HTTPS URL to .asc file>"
note          = "independent reproduction; matched hash"
```

This append-only addition is reviewed and merged. Multiple independent attestations from reviewers with different toolchains, kernels, and CPU microarchitectures are the load-bearing trust anchor.

## Step 6 — (Optional) verify against the published binary

```
curl -L "$binary_url" -o /tmp/published_stage0.bin
sha256sum /tmp/published_stage0.bin
diff <(xxd /tmp/published_stage0.bin) <(xxd pcc-stage0.bin)
```

A byte-for-byte match between the locally-built binary and the published one is the strongest confirmation. A hash match without a byte-diff is sufficient; a hash mismatch with the published binary, when the local build matches the recorded hash, indicates the published binary is corrupted — open an issue.

## What this procedure does *not* do

- It does not eliminate the trusted-trust gap. A malicious historical toolchain could have inserted a backdoor that propagates through every reproduction. Multiple-reviewer cross-checking with diverse toolchains mitigates but does not eliminate this.
- It does not bind future versions. A new stage 0 (e.g., for a new architecture) gets its own `[[stage0]]` entry in `bootstrap.toml` with its own provenance and its own attestation chain. The procedure is the same; the inputs differ.
- It does not certify anything. Phosphoric publishes no certifications. The attestations are evidence, not credentials.

## When this procedure becomes obsolete

Long-tail elevation item E15 (Reflections-on-Trusting-Trust hardening) replaces stage 0 with a bare-metal hand-authored stub that has no external-toolchain ancestry. When E15 lands and is the documented bootstrap path, this file gets archived alongside `STAGE0.md` as historical procedure. Until then, this is the runbook.
