# Public-archive provenance proof (commit → archive → deposit)

**Purpose.** Let any third party verify that a published archive (the Zenodo/arXiv tarball a reader downloads) is
*exactly* the tracked content of a specific git commit — no smuggled session backups, no controlled-access data, no
last-minute edits — and that it passes the same release-scrub gate the maintainer ran. This is the proof behind the
one-line recipe in [`release_checklist.md`](release_checklist.md) §C; that section has the commands, this one
explains the **chain of custody** and its honest limits.

## The chain

```
git commit SHA  ──(git archive, export-ignore rules)──▶  release tree  ──(SHA-256 manifest)──▶  deposit record
   (immutable)                                            (deterministic)        (publishable)     (Zenodo DOI)
```

1. **Commit SHA** — the immutable anchor. `git archive` materialises a tree that is a *pure function* of the commit
   and the `.gitattributes` `export-ignore` rules (the untracked `SESSION_*` backups and `research/` quarantine are
   excluded by those rules, so they cannot appear in the archive).
2. **Release tree** — `git archive HEAD | tar -x -C $DIR` reconstructs that tree. Run `release-scrub --archive-dir
   $DIR` on it (recipe in §C): it must report **RELEASE-CLEAN** (no placeholder DOI, hygiene config
   present, **no controlled-access rows**, controlled roles sidecars carry their no-rows flags).
3. **SHA-256 manifest** — the verifiable fingerprint of the release tree (see below).
4. **Deposit record** — publish the commit SHA + the manifest hash alongside the DOI so a downloader can re-derive
   and compare. (Depositing is **USER-ONLY**.)

## Producing the fingerprint

The robust, git-version-independent fingerprint is a **per-file SHA-256 manifest of the extracted tree**, not the
tarball's own byte-hash. The one-command form is **`bash scripts/build_public_archive.sh`** (it runs exactly the
steps below and prints `archive_manifest_sha256`); the manual recipe:

```fish
set TMP (mktemp -d)
git archive HEAD | tar -x -C $TMP
cargo run -q -p dsfb-chemical-engineering-edge -- release-scrub --archive-dir $TMP   # must be RELEASE-CLEAN
# deterministic content fingerprint of the shipped tree:
cd $TMP; find . -type f -exec sha256sum {} \; | LC_ALL=C sort | sha256sum
```

Record the commit SHA (`git rev-parse HEAD`) and that final `sha256` in the deposit notes. A verifier clones at the
commit, repeats the three commands, and confirms both match — proving the published bytes are the committed bytes.

## Honest limits

- **Tarball byte-hash is *not* the invariant.** `git archive`'s raw tar framing can differ across git versions
  (header padding, ordering details), so two correct archives of the same commit may have different *tarball* SHA-256
  values. The **extracted-tree manifest** above (sorted per-file hashes) is the stable invariant — use it, not the
  tarball hash, as the proof. (Pinning the git version makes the tarball hash reproducible too, but the manifest is
  the honest primary.)
- **It proves identity, not safety.** A clean scrub + matching manifest proves the archive *is* the committed tree
  and carries no flagged leak; it does not certify the code is correct or the data licensing is satisfied — those are
  the job of the test suite, the audit stack (`audit/`), and the dataset provenance manifest.
- **The maintainer still runs the outward step.** This document is a verification recipe; the actual deposit/upload
  is USER-ONLY ([`release_checklist.md`](release_checklist.md) §D).
