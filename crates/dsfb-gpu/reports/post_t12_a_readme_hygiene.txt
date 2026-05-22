================================================================
Post-T.12.a README hygiene — 10-step ritual receipt
================================================================

Context. Panel verdict 2026-05-16 surfaced four real issues with
the current archive after T.12.a sealed at `da1e9cf`. Two
require immediate fixing (stale README front-door wording +
T.12.0 sample hashes that understate determinism). Two are
future-work flags (UnknownSourceClass loader test; semantic
split of AcceptedProposalWithoutFutureFreezeGate via a
`ratification_commit` field). This commit ships ONLY the two
immediate fixes. No code changes, no test changes, no schema
changes, no hash mutation.

----------------------------------------------------------------
Panel findings (verbatim, panel-locked)
----------------------------------------------------------------

1. STALE README front matter (immediate fix). Line 18 still
   reads `**DSFB-GPU-Atlas (T.1–T.11h, S1.3a)**` but the archive
   now includes S1.3b, S1.3c, T.12.0, AND T.12.a. The Layout
   section (line 86) and the Paper section (line 226) carry the
   same stale framing. This is exactly the "front-door wording
   that hurts trust" the panel flagged.

2. README T.12.0 sample hashes UNDERSTATE determinism (immediate
   fix). The T.12.0 section said
   `literature_expansion_batch_hash_v1 : (per build; included in
   proposal hash)` and the same for `dedup_court_delta_hash_v1`
   when the actual emitted artifact carries stable concrete
   values. Calling them "per build" sounded less deterministic
   than the architecture actually is.

3. `UnknownSourceClass` verifier rule reserved for future TOML
   loader (future-work flag; honestly documented in the existing
   T.12.0 surface). No action this commit. When the first T.12.x
   commit ships an amendment-proposal file loader (TOML / JSON),
   add a real `proposal_loader_rejects_unknown_source_class`
   parser test.

4. `AcceptedProposalWithoutFutureFreezeGate` conflates
   `created_at_commit` with freeze authorization (future-work
   flag; defer to T.12.m consolidation). The panel-suggested
   split adds a `ratification_commit: Option<&'static str>`
   field to `CorpusAmendmentProposal` so the verifier can
   distinguish "filed at commit X" from "accepted for freeze at
   gate Y". Adding this field changes the proposal hash (would
   rebaseline T.12.0 + T.12.a); defer to T.12.m alongside the
   `corpus_hash_v2` preparation.

----------------------------------------------------------------
Scope (panel-locked, README + receipt only)
----------------------------------------------------------------

- README.md line 18 (front-door title): replace
  `**DSFB-GPU-Atlas (T.1–T.11h, S1.3a)**` with
  `**DSFB-GPU-Atlas (T.1–T.12.a, S1.3a–S1.3c)**`. Front-matter
  paragraph (lines 19–29) gains short acknowledgements of S1.3b
  (activation audit / diff), S1.3c (TaskManifest / DatasetManifest
  / ActivationContext), T.12.0 (amendment intake), and T.12.a
  (first real expansion proposal — SPC: MEWMA + MCUSUM
  canonicals, Q-stat / SPE / Hotelling T-sq aliases, Western
  Electric + Nelson composition reclassifications, four
  genealogy edges, four source refs — without mutating SEED or
  `corpus_hash_v1`).
- README.md Layout section (line 86): extend the corpus-crate
  description from "court + S1.3a activation planner; the Atlas
  frontier" to include S1.3b + S1.3c + T.12.0 + T.12.a.
- README.md Paper section (line 226): the parenthetical
  `(T.1–T.11h, S1.3a)` becomes `(T.1–T.12.a, S1.3a–S1.3c)` to
  match the front-door update.
- README.md T.12.0 sample hashes: replace
  `(per build; included in proposal hash)` with the actual
  stable values emitted by `dsfb-corpus
  amendment-proposal-emit`:
    literature_expansion_batch_hash_v1 =
      a57190d895c661b0c0f83ba64917ede3b97339f2b90c365e3b555dca432973ed
    dedup_court_delta_hash_v1            =
      6cf1c20e9a028d7c1fe98676c4604051cfc21303ec2943ce07218fb34e279c87

  The previously-pinned proposal hash itself
  (`325bbf3deff3595b429a3cda1d55a2fc9e31d689aaeb7a88e3bf8f691fc80092`)
  is now also written in full instead of `325bbf3d...`.

----------------------------------------------------------------
10-step ritual
----------------------------------------------------------------

Step 1. cargo fmt --all --check
  Result: clean.

Step 2. cargo clippy --workspace --all-targets --features cuda -- -D warnings
  Result: clean. Workspace pedantic lints unchanged from
  T.12.a seal — this commit makes no Rust edits.

Step 3. bash scripts/scrub.sh
  Result: clean. Zero attribution-string hits across the
  modified README front-matter / Layout / Paper / T.12.0
  sections.

Step 4. bash scripts/docs_freshness.sh
  Result: clean. Zero stale TODO / FIXME / XXX markers; zero
  missing-doc warnings under `cargo doc --no-deps
  --document-private-items` on the corpus crate (no Rust
  surface changed).

Step 5. cargo test -p dsfb-gpu-atlas-corpus
  Result: unchanged from the T.12.a seal — README-only edits
  do not exercise any test path. 742 corpus tests pass (no
  delta).

Step 6. cargo test --workspace --features cuda -- --test-threads=1
  Result: 66 test groups, 1057 tests PASS, 0 FAIL.
  Byte-identical to the T.12.a seal at `da1e9cf`.

Step 7. tests/r12_d64_saturation regression check
  Result: pinned episode counts byte-stable. Canonical 16x128
  K=1..128 -> episodes/cat=13; mid 64x512 -> episodes/cat=89;
  full 256x4096 -> episodes/cat=1917. Throughput fluctuates
  per run (documented GPU thermal / host-load variance); the
  pinned reports/r12_d64_saturation.txt baseline is NOT
  rebaselined.

Step 8. Stale-doc scan
  Result: clean. After the edit:
    grep -rEn 'T\.1.T\.11h, S1\.3a\)|T\.1.T\.11h, S1\.3a\*\*'
      --include='*.md' --include='*.rs' --include='*.tex'
      --include='*.toml' .
  returns ZERO hits across the repo (excluding target/ and .git/).
  The paper's Atlas Continuation paragraph
  still mentions T.11h alongside S1.3a–T.12.a as an accurate
  enumeration of every sealed surface, which is correct (not
  stale).

Step 9. README / plan refresh
  Result:
    - README.md: this hygiene commit's five-spot rewrite.
    - Plan-file: the post-T.12.a README hygiene section was
      added in the previous Plan Mode turn and authorized.
      Post-commit plan touch-up will mark the hygiene commit
      sealed and surface T.12.b as the next campaign.

Step 10. Atomic commit
  Single atomic commit landing:
    - README.md (5 stale spots fixed; no other changes)
    - reports/post_t12_a_readme_hygiene.txt (this receipt)
    - .gitignore (1 new whitelist line for the receipt)
  No --no-verify. Pre-commit hook gates pass.

----------------------------------------------------------------
Hash posture (panel-locked, MUST hold)
----------------------------------------------------------------

- corpus_hash_v1                              : unchanged (35c276c7...)
- registry_hash_v2                            : unchanged (d3cf6300...)
- precedent_hash_v1                           : unchanged (6721f511...)
- admissibility_grammar_hash_v1               : unchanged (ff66706a...)
- trial_transcript_hash_v1                    : unchanged (37618a45...)
- execution_attestation_receipt_hash_v1       : unchanged
- challenge_docket_hash_v1                    : unchanged (dde4ecb4...)
- detector_contraindication_hash_v1           : unchanged (1b899f5d...)
- coverage_hole_hash_v1                       : unchanged (671e2164...)
- activation_plan_hash_v1                     : unchanged (5a81da47...)
- activation_diff_hash_v1                     : unchanged (469536e7...)
- task_manifest_hash_v1                       : unchanged (88a33338...)
- dataset_manifest_hash_v1                    : unchanged (3864db34...)
- activation_context_hash_v1                  : unchanged (4948bf45...)
- T.12.0 proof-of-life proposal hash          : unchanged (325bbf3d...)
- T.12.a SPC corpus_amendment_proposal_hash_v1: unchanged (ae493a85...)
- Every DetectorPassport hash                 : unchanged
- SEED.len()                                  : 54 (unchanged)
- R.12b episodes 13/89/1917                   : byte-stable

ZERO hash mutation. No schema-byte change, no code change.

----------------------------------------------------------------
Doctrine constraints preserved
----------------------------------------------------------------

- No probability, no learned weights, no fast-math, no atomics
  for accumulation.
- Semantic Non-Bypass Axiom: only the bank module's private
  BankAdmissionToken constructor mints admitted episodes.
- Apache-2.0 + Invariant Forge LLC Background IP notice; no
  MIT regression.
- No attribution-string regressions (scrub clean).
- Audit mode untouched; D16 / D64 / D128 / D205 GPU behaviour
  unchanged; the corpus crate stays host-only and zero-dep.
- [[no-silent-court-logic]] doctrine: no Rust surface changed,
  so the existing WHY commentary on every pub item + private
  helper remains unaffected.

----------------------------------------------------------------
Next campaign (panel-locked)
----------------------------------------------------------------

T.12.b - Sequential Change Detection. Same T.12.0 amendment
scaffold; cross-class authority resolution for Page-Hinkley
(SEED id 4) / CUSUM (SEED id 3) / Mann-Kendall (SEED id 11);
new canonical primitives reserved at ids 5201+ (Shiryaev-
Roberts, GLR, Pettitt, Buishand range, SNHT, MOSUM, Binary
segmentation, PELT-style deterministic); BOCPD admitted as a
RejectedNotDeterministic dedup record; minimum 7 panel-required
load-bearing negatives. Hash posture identical to T.12.a:
corpus_hash_v1 + every T.11/S1.3/T.12.0/T.12.a hash byte-
identical; SEED.len() stays at 54; corpus_hash_v2 NOT created.
