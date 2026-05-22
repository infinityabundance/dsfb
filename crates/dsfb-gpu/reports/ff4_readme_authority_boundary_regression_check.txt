================================================================
FF.4 - README authority-boundary policy
       regression-check receipt (10-step ritual)
================================================================

Step 1. cargo fmt --all --check
  Result: clean.

Step 2. cargo clippy --workspace --all-targets --features cuda -- -D warnings
  Result: clean. Initial pass hit:
  - items_after_statements on three inline `const` arrays of
    forbidden-substring phrases inside `verify_ff4_readme`
    (`STALE_FUTURE_RATIFICATION_PHRASES`,
    `T12_MUTATED_SEED_PHRASES`,
    `FF1_MUTATED_CORPUS_V2_PHRASES`). Resolved by hoisting all
    three to module-scope `const` declarations alongside the
    domain-separator constants. Hash material unchanged
    (`ff4_readme_authority_boundary_policy_hash_v1` =
    `22b9dcb5...` byte-identical across hoist).

Step 3. bash scripts/scrub.sh
  Result: clean.

Step 4. bash scripts/docs_freshness.sh
  Result: clean.

Step 5. cargo test -p dsfb-gpu-atlas-corpus
  Result: 47 groups / 1643 tests pass (FF.3 baseline
  46 groups / 1601 tests + 1 new group + 42 new
  ff4_readme_authority_boundary_invariants tests). The 42
  new tests include the 7 panel-required load-bearing
  negatives, the live README sweep
  (`ff4_live_readme_satisfies_policy` reads README.md from
  disk), every determinism + sensitivity invariant, every
  upstream-anchor invariance witness, every field-level +
  wire-name + domain-separator pin, every block-coverage
  invariant, every hash-namespace distinctness assertion, and
  every renderer-coverage check.

Step 6. cargo test --workspace --features cuda -- --test-threads=1
  Result: 86 test groups, 1958 tests PASS, 0 FAIL. Delta from
  FF.3 seal (fa5f214): +1 test group, +42 tests. No known-flake
  retries needed.

Step 7. tests/r12_d64_saturation regression check
  Result: pinned episode counts byte-stable (13/89/1917).
  Pinned R.12b baseline NOT rebaselined. The saturation test
  rewrites reports/r12_d64_saturation.txt as part of its run;
  the pinned baseline was restored from HEAD-equivalent backup
  before commit (also for reports/d64_stage_timing_256x4096_K1.txt,
  reports/r9_b3_d64_timing.txt, reports/r9_b_d64_timing.txt).

Step 8. Stale-doc scan
  Result: clean. src/lib.rs module docstring header refreshed
  through FF.4; new FF.4 enumeration bullet inserted after
  FF.3; CLI list extended with 3 new ff4-policy / ff4-policy-emit
  / ff4-authority-boundary-block subcommands. README +
  paper get extended in this same commit.

Step 9. README / plan refresh
  Result: README gets the canonical 19-line authority-boundary
  block inserted immediately after the Panel-locked-anchor
  block and before `## What this is`; front-door title bumped
  through FF.4; test count bumped to 1643/47 groups; paper
  section enumeration bumped; new FF.4 section between FF.3
  and the scaling-ladder header; Layout block extended with
  FF.4 descriptor. Plan-file post-commit will mark FF.4 sealed
  and rotate the Quick Start "Active campaign (next)" anchor
  to FF.5 (ProposalSchemaUpgradePolicy).

Step 10. Atomic commit
  Single atomic commit landing:
    - src/ff4_readme_authority_boundary.rs (new, ~700 lines
      fully commented: canonical 19-line authority-boundary
      block + 6-entry required-substring set + 7-entry
      forbidden-substring set + verifier with 12 reject kinds
      (7 panel-required negatives + 5 structural rules + SEED
      invariance) + text + JSON renderers + canonical-block
      renderer)
    - src/lib.rs (module export + docstring header bump to
      "Current state through FF.4" + new FF.4 enumeration
      bullet + 3 new CLI list entries)
    - src/main.rs (3 CLI subcommands: ff4-policy{,-emit} +
      ff4-authority-boundary-block + usage block)
    - tests/ff4_readme_authority_boundary_invariants.rs (new,
      42 tests including 7 panel-required negatives + live
      README sweep)
    - out/ff4_readme_authority_boundary_policy_v1.{txt,json}
      (new)
    - reports/ff4_readme_authority_boundary_summary.txt (new)
    - reports/ff4_readme_authority_boundary_verification.txt
      (new)
    - reports/ff4_readme_authority_boundary_regression_check.txt
      (new)
    - .gitignore (3 new whitelist lines)
    - README.md (new "## Authority boundary" 19-line block
      inserted right after the Panel-locked-anchor block +
      front-door + test-count + paper-section + Layout block
      extension + new FF.4 section between FF.3 and the
      scaling-ladder header)
    - archived prior-art manuscript source (lead sentence + paragraph
      header + new \item \textbf{FF.4} bullet)
  No --no-verify. Pre-commit hook gates pass.

Hash posture:
  corpus_hash_v1                              : unchanged
                                                (35c276c7...)
  corpus_hash_v2                              : unchanged
                                                (f1d132eb...)
  consolidation_report_hash_v1                : unchanged
                                                (2842f6ae...)
  t12_expansion_index_hash_v1                 : unchanged
                                                (11fe6543...)
  ff1_passport_index_hash_v1                  : unchanged
                                                (1ad2dc2d...)
  ff1_materialisation_report_hash_v1          : unchanged
                                                (5edacbc4...)
  ff2_activation_ratification_gate_hash_v1    : unchanged
                                                (05c1b552...)
  ff2_activation_ratification_gate_summary_hash_v1 : unchanged
                                                     (e671cfc0...)
  ff3_registry_generation_gate_hash_v1        : unchanged
                                                (2ffd0222...)
  ff3_registry_generation_gate_summary_hash_v1 : unchanged
                                                 (c66f8174...)
  Every prior T.11/S1.3/T.12.0-T.12.p hash    : unchanged
  Every T.11a SEED-DetectorPassport hash      : unchanged
  Every T.12.consolidate hash                 : unchanged
  Every FF.1 per-passport passport_hash_v1    : unchanged
  SEED.len()                                  : 54 (unchanged)

  FF.4 NEW ff4_readme_authority_boundary_policy_hash_v1 :
    22b9dcb57037d45983de27ca6e50c35b2c2053efe23cb961ca4e014eaf963949
    Domain: DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1\0

Doctrine constraints preserved:
- No probability, no learned weights, no fast-math, no atomics
  for accumulation.
- Semantic Non-Bypass Axiom intact.
- Apache-2.0 + Invariant Forge LLC Background IP notice; no
  MIT regression.
- No attribution-string regressions.
- Corpus crate stays host-only and zero-dep.
- no-silent-court-logic doctrine: every pub item AND every
  private helper in ff4_readme_authority_boundary.rs carries
  a doc comment whose first sentence states the WHY for a
  future engineer.
- Panel-locked discipline: FF.4 is a COMMUNICATION-HYGIENE
  seal, not an authority mutation. It does NOT add new
  detectors, does NOT mutate any upstream hash, does NOT
  promote any open proposal to Accepted, does NOT change
  S1.3a / FF.2 / FF.3 court decisions, does NOT itself emit
  DetectorSpec records, does NOT modify dsfb-gpu-atlas-registry's
  existing 162-spec registry_hash_v2, does NOT decide
  contraindications or challenges, does NOT generate CUDA
  kernels.

Panel-locked one-line verdict enforced:
  > FF.4 makes the authority boundary unmissable at the front
  > door; it does not move any boundary.

FF.4 is sealed. The README front door now carries the
canonical post-T.12.consolidate / post-FF.1 / post-FF.2 /
post-FF.3 authority-boundary block, and the live README
content is verified test-side against the policy on every
build.

Next campaign (panel-authorized post-FF.4): FF.5
ProposalSchemaUpgradePolicy. Defines how proposal schema
upgrades may re-render historical proposal artifacts without
erasing the old artifact hashes or confusing the court
lineage. Core rule: schema upgrade != silent artifact
rewrite. After FF.5: S1.3d budget pruning + redundancy
suppression begins.
