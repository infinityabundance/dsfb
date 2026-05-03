# Phase 0 Hand-Bootstrap Runbook (External Attesters)

This file is for external attesters. The active repo never invokes any external toolchain. Phase 0 binary production happens **outside** the active repo, by attesters following these steps once. The resulting binary is hash-pinned in [bootstrap.toml](../bootstrap/bootstrap.toml) and never re-built unless the published phase 0 source changes.

`make verify` does not run any of these steps.

## Preconditions

A phase 0 attester needs:

1. A clean Linux x86_64 host (fresh container or VM is preferred; environmental drift is the leading cause of binary non-determinism).
2. A copy of [phase0/phase0_compiler.phos](phase0_compiler.phos) at the published commit.
3. A copy of [compiler/pcc.phos](../compiler/pcc.phos) and the modules it imports, at the same commit.
4. The published [phase0/phase0_subset.md](phase0_subset.md) spec, identical bytes.
5. A trusted GPG keyring containing the original attester's public key (for cross-attestation only — first attester is self-signed).
6. **No external toolchain.** Attesters hand-author x86_64 ASM. This is the doctrine: the active repo never depends on any compiler other than its own.

## Step 1 — Read the subset spec

Read [phase0_subset.md](phase0_subset.md) end-to-end. Internalize the accepted surface. The hand-coded ASM you write will only need to support exactly what phase0_subset.md lists.

## Step 2 — Hand-author the x86_64 ASM stub

Your task: write x86_64 assembly that can:

1. Read a `.phos` source file by path (use Linux syscalls: `open`, `read`, `close`).
2. Lex it into a token stream per the subset spec's lexer rules.
3. Parse the tokens into an AST.
4. Lower the AST to a tiny IR (just enough to emit ELF code).
5. Emit a Linux x86_64 ELF executable (use the ELF header layout pinned in [docs/BOOT_ABI_V1.md](../docs/BOOT_ABI_V1.md) §host).
6. Exit 0 on success; non-zero on subset violation with a P0-### code printed to stderr.

The ASM stub is yours alone. Multiple attesters write multiple ASM stubs.

The ASM stub is **not** in the active repo and never enters it. Keep it on a separate medium; its only role is producing the binary in step 4.

## Step 3 — Produce stage 1 source bundle

Concatenate (in deterministic order) the source bytes of `compiler/pcc.phos` and every `compiler/*.phos` module it directly or transitively imports. The order is alphabetical by module path. The bundle is the input to phase 0.

Hash the bundle with SHA-256. This hash is the "source bundle hash" recorded in [bootstrap.toml](../bootstrap/bootstrap.toml)'s `[stage0.source_provenance].source_bundle_hash` field.

## Step 4 — Run phase 0 ASM stub against phase0_compiler.phos

```
./your-handwritten-asm-stub phase0/phase0_compiler.phos -o phase0_compiler.bin
```

This produces an x86_64 ELF executable, `phase0_compiler.bin`, that is the phase 0 compiler.

## Step 5 — Run phase 0 compiler against pcc.phos

```
./phase0_compiler.bin compiler/pcc.phos -o pcc-stage1.bin
```

This produces stage 1: a self-hosted `pcc.phos` binary. SHA-256 it.

## Step 6 — Verify the fixpoint

```
./pcc-stage1.bin compiler/pcc.phos -o pcc-stage2.bin
./pcc-stage2.bin compiler/pcc.phos -o pcc-stage3.bin
sha256sum pcc-stage2.bin pcc-stage3.bin
```

Stage 2 and stage 3 MUST hash byte-identical. This is the convergence proof: stage 1 (built by phase 0) compiles `pcc.phos` correctly because compiling `pcc.phos` with the result is a fixpoint.

If stage 2 ≠ stage 3, the chain is broken. Investigate before attesting.

## Step 7 — Attest

### Quick path (Pass R reproduction harness)

For attesters reproducing the **current pinned phase-0 binary** (without yet
exercising the full stage-1 → stage-2 → stage-3 fixpoint chain — which
requires Pass M.3+ in the producer), the project ships an automation
script:

```
bash untracked/internaldocs/phase0_producer/attest_repro.sh "<your attestor label>"
```

The script:
1. Verifies that `phase0/phase0_compiler.phos` is present and hashes the
   exact source bytes.
2. Re-runs the hand-coded ASM stub (your own, per Step 2; or the scaffold
   stub) against the same source.
3. Compares the produced binary's SHA-256 against the manifest's pinned
   `binary_sha256`.
4. On match, prints a paste-ready `[[stage0.attestation]]` block. On
   mismatch, fails closed with the expected vs. actual values.

**Currently pinned input/output (manifest values as of Pass L):**
- Source: `phase0/phase0_compiler.phos`
- Source SHA-256 (from your local checkout — the script will print it
  fresh, do not trust this README's copy if they disagree)
- Expected binary size: 952 bytes (52 top-level fns × 16 + 120-byte
  ELF header + program header).
- Expected binary SHA-256: `790e531726de401d06d917dc914a1c4887374b65377e165c5d18a290c09bb368`

If your hand-authored ASM stub produces a binary whose hash does NOT match
the pinned value but the source bytes are identical, that is meaningful
data — record both hashes and submit the discrepancy as a separate audit
trail entry. Do NOT submit a non-matching attestation as if it matched.

### Manual path (full fixpoint chain, when Pass P lands)

When the producer can compile `compiler/pcc.phos` into a real compiler
(Pass M.3 + N + O + P; multi-week work documented in
`bootstrap/STAGE0_STATUS.md`), the chain runs end-to-end:

```
sha256sum pcc-stage1.bin > stage1.sha256
gpg --detach-sign --armor stage1.sha256
```

Submit a PR adding to [bootstrap.toml](../bootstrap/bootstrap.toml):

```toml
[[stage0.attestation]]
attestor       = "<your GPG fingerprint>"
signed_at      = "<ISO-8601 timestamp>"
signature_url  = "<HTTPS URL to your .asc>"
note           = "independent reproduction; matched stage 2 == stage 3 fixpoint"
```

## What this procedure does *not* do

- Eliminate the trusted-trust gap. Your hand-authored ASM could contain a backdoor; multiple attesters with different ASM stubs and different machines mitigate but do not eliminate this.
- Bind future versions. A new phase 0 (e.g., for a new architecture, or after a phase0_compiler.phos source change) is a new attestation cycle.
- Certify. Phosphoric publishes no certifications. Attestations are evidence.

## When this procedure becomes obsolete

When the long-tail item to replace phase 0 with an OTP-fused boot ROM stub (no software-mediated trust at all) lands, this file becomes historical. Until then, this is the runbook.
