# Release / deposit checklist — DSFB-Chemical-Engineering

> The repeatable gate before any public release or deposit. **Everything outward-facing is USER-ONLY** and is
> marked USER-ONLY below — the local release operator prepares artifacts and runs local gates, but never pushes,
> deposits, publishes, or contacts anyone. Run the local gates first; do the USER-ONLY steps yourself.

## A. Local verification gates (run all; all must be green)
```bash
cargo test --workspace                                            # all tests, 0 failed
cargo run -q -p dsfb-chemical-engineering-edge -- verify-replay   # 6/6 byte-identical
cargo run -q -p dsfb-chemical-engineering-edge -- completeness-court        # COMPLETE
cargo run -q -p dsfb-chemical-engineering-edge --features soft-sensor-corpus -- completeness-court  # 9/0
cargo run -q -p dsfb-chemical-engineering-edge -- release-scrub             # RELEASE-CLEAN
cargo run -q -p dsfb-chemical-engineering-edge -- unit-consistency          # all balances consistent
python3 scripts/verify_reproducibility.py --bundles                         # 20/20 bundle/evidence roots
bash paper/build_paper.sh                                                    # 0 overfull / 0 undefined
# Optional GPU path (needs nvcc + an NVIDIA GPU):
#   CUDA_HOME=/opt/cuda PATH=/opt/cuda/bin:$PATH cargo test --release -p dsfb-chemical-engineering-cuda --features cuda
# Optional embedded smoke (needs qemu-system-arm + rustup target add thumbv7m-none-eabi):
#   (cd crates/dsfb-chemical-engineering-core/qemu-smoke && cargo run --release)
```
The `release-scrub` court is the machine release-blocker: it fails on a placeholder DOI, a leaked private
backup, a controlled-access row, or a missing required artifact. Do not release if it fails. It has two
modes: the default (no args) scrubs the **tracked tree** (and verifies the session backups are git-ignored +
export-ignored); `release-scrub --archive-dir <dir>` (P81) scrubs a **materialised archive** and fails hard if
a session backup actually shipped — run it on the staged `git archive` output (section C) before uploading.

## B. Metadata + reproducibility surface (verify present + current)
- [ ] `reports/verification_report.md` (top-level) test counts match the actual run.
- [ ] `CITATION.cff` version + title + DOI current.
- [ ] `Dockerfile` builds and `docker run --rm dsfb-chem-edge verify-replay` is 6/6.
- [ ] Frozen authority hashes unchanged unless a governed re-freeze is noted in `PROJECT_PLAN.md`
      + `data/EXPECTED_BUNDLE_ROOTS.toml` (`atlas_hash_v1`, `corpus_hash_v1`, bundle/evidence roots).
- [ ] `.gitattributes` export-ignores the untracked session backups (`git archive` excludes them).

## C. Release-tarball hygiene
- [ ] `git archive` (not a raw tar/zip) so export-ignored files are excluded; spot-check the tarball has no
      `SESSION_*` backups and no `research/`.
- [ ] **Run the archive-mode scrub court on the staged archive before uploading** — this is the machine gate
      for THIS exact failure mode (a raw zip that smuggled git-ignored backups in):
      ```fish
      set TMP (mktemp -d); git archive HEAD | tar -x -C $TMP
      cargo run -p dsfb-chemical-engineering-edge --bin dsfb-chem-edge -- release-scrub --archive-dir $TMP
      # must print RELEASE-CLEAN; it FAILS on any SESSION_*/SESSION_FULL_CONTEXT_* in the tree, a missing
      # .gitignore/.gitattributes, a controlled-access row, or a placeholder DOI. Then upload $TMP (or its tarball).
      rm -rf $TMP
      ```
- [ ] No real/gated plant data in the tree (only recipes + SHA-256 digests are committed; SWaT/BATADAL raw
      data is never redistributed).
- [ ] **Generate the public-archive proof** in one step (does the materialise + scrub + fingerprint above and
      writes a recordable proof report):
      ```fish
      bash scripts/build_public_archive.sh        # → RELEASE-CLEAN + reports/public_archive_proof.md
      ```
      Record the `commit` + `archive_manifest_sha256` from `reports/public_archive_proof.md` in the deposit
      notes (alongside the DOI). The proof file is git-ignored (a tree-fingerprint cannot live in the tree it
      hashes) — regenerate it at deposit; a verifier re-runs the script and compares the manifest hash.
- [ ] **Regenerate + verify the artifact index** so the committed `reports/index.{html,json}` is current:
      ```fish
      cargo run -p dsfb-chemical-engineering-edge -- generate-index   # rebuild the index
      cargo run -p dsfb-chemical-engineering-edge -- verify-index     # → INDEX-VERIFIED (court)
      ```

## D. 🔌 USER-ONLY outward steps (the local release operator never does these)
- [ ] 🔌 `git push` to the public remote.
- [ ] 🔌 Zenodo deposit (DOI `10.5281/zenodo.20443279` already in `CITATION.cff` + the paper) → re-tag.
- [ ] 🔌 arXiv submission of the paper, only after the paper source, PDF, and verification log are committed together.
- [ ] 🔌 `cargo publish` (per crate) to crates.io, if publishing the libraries.
- [ ] 🔌 `maturin publish` for the Python bindings, if shipping a wheel.
- [ ] 🔌 Any integrator outreach, commercial licensing, or live-plant pilot.

## Honest scope reminder
The local gates assert the machine-checkable artifact graph on the maintainer's host (see
`reports/verification_report.md`). Full real-plant TRL (4→5) depends on a user-supplied real ungated historian
export (`docs/real_data_dropin.md`, risk R1). Advisory, read-only, no control/safety authority throughout.
