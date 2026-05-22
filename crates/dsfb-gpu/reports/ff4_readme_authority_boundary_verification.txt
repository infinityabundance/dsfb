================================================================
FF.4 - README authority-boundary policy
       verification receipt
================================================================

Verifier coverage table — every reject kind, the rule it
encodes, the panel-required negative or structural rule it
satisfies, and the acceptance test that exercises it.

----------------------------------------------------------------
Panel-required load-bearing negatives (7)
----------------------------------------------------------------

R.1  StaleFutureRatificationLanguage
     Reject any README containing the pre-ratification phrases
     ("future ratification / freeze campaign", "until a future
     ratification campaign", "until a future freeze"). Those
     were correct before T.12.consolidate; they became
     operator-misleading after T.12.consolidate + FF.1.
     Tests:
       ff4_readme_rejects_stale_future_ratification_language
       ff4_readme_rejects_each_stale_phrase_variant

R.2  MissingCorpusHashV1HistoricalAnchorLanguage
     Reject any README lacking the phrase "historical
     seed-corpus anchor" — the operator-facing language that
     identifies corpus_hash_v1 as the historical anchor
     post-ratification.
     Test: ff4_readme_requires_corpus_hash_v1_historical_anchor_language

R.3  MissingCorpusHashV2RatifiedAuthorityLanguage
     Reject any README lacking the phrase "ratified
     post-amendment authority" — the operator-facing language
     that identifies corpus_hash_v2 as the ratified
     post-amendment corpus authority.
     Test: ff4_readme_requires_corpus_hash_v2_ratified_authority_language

R.4  MissingFf1PassportMaterialisationLanguage
     Reject any README lacking the phrase "FF.1 materialized
     ratified T.12 additions into passports.".
     Test: ff4_readme_requires_ff1_passport_materialisation_language

R.5  MissingFf2Ff3UnratifiedRejectionLanguage
     Reject any README lacking the phrase "FF.2 / FF.3 prevent
     unratified records from entering activation or registry
     generation.".
     Test: ff4_readme_requires_ff2_ff3_unratified_rejection_language

R.6  ClaimThatT12ProposalsMutatedSeed
     Reject any README claiming T.12 proposals mutated SEED.
     They did not.
     Test: ff4_readme_rejects_claim_that_t12_proposals_mutated_seed

R.7  ClaimThatFf1MutatedCorpusHashV2
     Reject any README claiming FF.1 mutated corpus_hash_v2.
     FF.1 emits its own per-passport / index / report hashes;
     corpus_hash_v2 stays byte-identical.
     Test: ff4_readme_rejects_claim_that_ff1_mutated_corpus_hash_v2

----------------------------------------------------------------
Structural sweep rules (general substring-set guards)
----------------------------------------------------------------

R.8  MissingRequiredSubstring (catch-all)
     Reject when any entry of FF4_REQUIRED_SUBSTRINGS is
     missing from the README. Surfaces required-substring
     drift beyond the specific anchors R.2..R.5 protect.

R.9  ForbiddenSubstringPresent (catch-all)
     Reject when any entry of FF4_FORBIDDEN_SUBSTRINGS appears
     in the README beyond the specific phrases R.1 / R.6 / R.7
     already cover.

R.10 CorpusHashV1Mismatch
     Reject when the policy's pinned corpus_hash_v1 does not
     equal compute_corpus_hash_v1().
     Cross-checked by ff4_policy_pins_live_corpus_hash_v1.

R.11 CorpusHashV2Mismatch
     Reject when the policy's pinned corpus_hash_v2 does not
     equal the live consolidation report's corpus_hash_v2.

R.12 SeedLengthMutated (unconditional)
     Reject when SEED.len() != 54.

----------------------------------------------------------------
Live README sweep (the hygiene seal)
----------------------------------------------------------------

  ff4_live_readme_satisfies_policy reads README.md from disk
  and verifies the entire policy: every required substring
  present, every forbidden substring absent, every pinned
  anchor matching live state. This is the regression sentinel
  that prevents future commits from regressing the front-door
  authority-state story.

----------------------------------------------------------------
Determinism + sensitivity + invariance invariants
----------------------------------------------------------------

  ff4_policy_hash_is_deterministic_across_two_builds
  ff4_policy_text_render_byte_stable
  ff4_policy_json_render_byte_stable
  ff4_authority_boundary_block_render_byte_stable
  ff4_does_not_mutate_corpus_hash_v1
  ff4_does_not_mutate_corpus_hash_v2
  ff4_does_not_mutate_ff1_passport_index_hash_v1
  ff4_does_not_mutate_ff2_gate_hash
  ff4_does_not_mutate_ff3_gate_hash
  ff4_does_not_mutate_seed_len

----------------------------------------------------------------
Pinned anchor cross-checks
----------------------------------------------------------------

  ff4_policy_pins_live_corpus_hash_v1
  ff4_policy_pins_live_corpus_hash_v2
  ff4_policy_pins_live_ff1_passport_index_hash_v1
  ff4_policy_pins_live_ff2_gate_hash
  ff4_policy_pins_live_ff3_gate_hash

----------------------------------------------------------------
Pinned constants / disjoint-set discipline
----------------------------------------------------------------

  ff4_domain_separator_pin
  ff4_schema_pin
  ff4_block_lines_non_empty
  ff4_required_substrings_non_empty
  ff4_forbidden_substrings_non_empty
  ff4_required_and_forbidden_substring_sets_are_disjoint

----------------------------------------------------------------
Block-coverage invariants
----------------------------------------------------------------

  ff4_canonical_block_contains_all_required_substrings
  ff4_canonical_block_contains_no_forbidden_substring
  ff4_block_lines_pin_first_line_is_header
  ff4_block_line_count_is_nineteen

----------------------------------------------------------------
Hash-namespace distinctness
----------------------------------------------------------------

  ff4_policy_hash_distinct_from_corpus_hash_v1
  ff4_policy_hash_distinct_from_corpus_hash_v2
  ff4_policy_hash_distinct_from_ff1_passport_index_hash
  ff4_policy_hash_distinct_from_ff2_gate_hash
  ff4_policy_hash_distinct_from_ff3_gate_hash

----------------------------------------------------------------
Render coverage
----------------------------------------------------------------

  ff4_render_text_contains_pinned_anchors_and_block
  ff4_render_json_contains_schema_field

----------------------------------------------------------------
Summary
----------------------------------------------------------------

  Total FF.4 acceptance tests              : 42
  Panel-required load-bearing negatives    : 7  (R.1..R.7)
  Structural / determinism / sensitivity   : 12
  Default-build invariants                 : 2
  Field-level / pin / anchor cross-checks  : 12
  Block-coverage invariants                : 4
  Hash-namespace distinctness assertions   : 5
  Render coverage assertions               : 2
  Live README sweep                        : 1

  Corpus crate test count change           : 1601 -> 1643
                                            (46 -> 47 groups)
  Workspace serial test count change       : 1916 -> 1958
                                            (85 -> 86 groups)
