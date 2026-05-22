================================================================
FF.4 - README authority-boundary policy
       summary receipt
================================================================

Panel-locked opening guard. FF.4 makes the post-T.12.consolidate
/ post-FF.1 / post-FF.2 / post-FF.3 authority-boundary state
unmissable at the README front door. It does not add detectors,
mutate any upstream hash anchor, modify SEED, modify any court
artifact, or change activation / registry-generation behaviour.
It is a communication-hygiene seal: the operator-facing README
MUST carry the canonical authority-boundary block stating that
T.12.a..T.12.p were filed as amendment proposals (and did not
mutate seed authority while filed), that T.12.consolidate froze
corpus_hash_v2 as the ratified post-amendment authority, that
FF.1 materialized 98 ratified CanonicalAddition entries into
T12RatifiedPassport records, and that FF.2 + FF.3 reject
unratified / non-passported / ad-hoc / unknown-source records
by explicit reason code.

Panel-locked one-line verdict (verbatim):

> FF.4 makes the authority boundary unmissable at the front
> door; it does not move any boundary.

----------------------------------------------------------------
One new own-namespace hash layer (panel-locked)
----------------------------------------------------------------

FF.4 emitted value:

  ff4_readme_authority_boundary_policy_hash_v1 :
    22b9dcb57037d45983de27ca6e50c35b2c2053efe23cb961ca4e014eaf963949
    Domain:
    DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1\0
    META-hashes the canonical authority-boundary block + the
    required + forbidden substring sets + the five pinned
    upstream anchor hashes.

Distinct from every prior T.11 / S1.3 / T.12.x /
T.12.consolidate / FF.1 / FF.2 / FF.3 hash.

----------------------------------------------------------------
Pinned upstream anchors (FF.4 does NOT mutate any of these)
----------------------------------------------------------------

  corpus_hash_v1                              :
    35c276c73a52d916daafda2598b215d73e7fd694d4a0673e34ac1ef948f5a4b7
  corpus_hash_v2                              :
    f1d132eba43795c3087f1e388ba040357c6a0779fe22bd29de24c885ca98383f
  ff1_passport_index_hash_v1                  :
    1ad2dc2d9137320b942c17a6f5c8440b5caad03de5a92c9ae5d88f3069db2717
  ff2_activation_ratification_gate_hash_v1    :
    05c1b552652c321dc670628207624d1afb1e9fcb4ea15a028e89b80ff7899efb
  ff3_registry_generation_gate_hash_v1        :
    2ffd02229a4e0f99023a5791ed27e097346d29c14c9eeebd7a2ee8d26caf0d41
  SEED.len()                                  : 54 (unchanged)

----------------------------------------------------------------
Canonical authority-boundary block (19 lines, pinned verbatim)
----------------------------------------------------------------

  ## Authority boundary (post-T.12.consolidate + FF.1 + FF.2 + FF.3)
  Important authority-state note. T.12.a..T.12.p were amendment proposals.
  They did not mutate SEED, corpus_hash_v1, registry_hash_v2, historical
  DetectorPassports, or activation outputs while they were filed.
  T.12.consolidate ratified the accepted T.12 expansion set and froze
  corpus_hash_v2 as the first post-amendment corpus authority.
  FF.1 then materialized 98 ratified T.12 CanonicalAddition entries into
  T12RatifiedPassport records under ff1_passport_index_hash_v1.
  FF.2 and FF.3 now enforce that activation and registry generation consume
  only SeedHistorical records or T12RatifiedAndPassported records. Unratified,
  non-passported, ad-hoc, or unknown-source records are rejected by explicit
  reason code ...
  - SEED and corpus_hash_v1 remain the historical seed-corpus anchor.
  - T.12 proposals did not mutate seed authority while filed.
  - T.12.consolidate froze corpus_hash_v2 as ratified post-amendment authority.
  - FF.1 materialized ratified T.12 additions into passports.
  - FF.2 / FF.3 prevent unratified records from entering activation or registry generation.

(Full block in
crates/dsfb-gpu-atlas-corpus/out/ff4_readme_authority_boundary_policy_v1.txt.)

----------------------------------------------------------------
Required substrings (6, pinned)
----------------------------------------------------------------

  + historical seed-corpus anchor
  + ratified post-amendment authority
  + T12RatifiedPassport
  + FF.2 / FF.3 prevent unratified records from entering activation or registry generation.
  + FF.1 materialized ratified T.12 additions into passports.
  + T.12.consolidate froze corpus_hash_v2

----------------------------------------------------------------
Forbidden substrings (7, pinned)
----------------------------------------------------------------

  - future ratification / freeze campaign
  - until a future ratification campaign
  - until a future freeze
  - T.12 proposals mutated SEED
  - T.12 proposals mutate SEED
  - FF.1 mutated corpus_hash_v2
  - FF.1 mutates corpus_hash_v2

----------------------------------------------------------------
Seven panel-required load-bearing negatives (acceptance suite)
----------------------------------------------------------------

  ff4_readme_rejects_stale_future_ratification_language
  ff4_readme_requires_corpus_hash_v1_historical_anchor_language
  ff4_readme_requires_corpus_hash_v2_ratified_authority_language
  ff4_readme_requires_ff1_passport_materialisation_language
  ff4_readme_requires_ff2_ff3_unratified_rejection_language
  ff4_readme_rejects_claim_that_t12_proposals_mutated_seed
  ff4_readme_rejects_claim_that_ff1_mutated_corpus_hash_v2

Plus a live README sweep
(`ff4_live_readme_satisfies_policy`) that reads the on-disk
`README.md` and verifies every required substring is present
and every forbidden substring is absent. This is the hygiene
seal that prevents future commits from regressing the
front-door authority-state story.

----------------------------------------------------------------
README change (the actual communication seal)
----------------------------------------------------------------

The canonical 19-line authority-boundary block is now inserted
in `README.md` immediately after the panel-locked-anchor
block and before the `## What this is` section, exactly at the
front-door area an operator first encounters.

----------------------------------------------------------------
Panel-locked non-claims (must NOT regress)
----------------------------------------------------------------

  - FF.4 does NOT add new detectors.
  - FF.4 does NOT alter corpus_hash_v1, corpus_hash_v2,
    consolidation_report_hash_v1, t12_expansion_index_hash_v1,
    ff1_passport_index_hash_v1, ff1_materialisation_report_hash_v1,
    ff2_activation_ratification_gate_hash_v1, or
    ff3_registry_generation_gate_hash_v1.
  - FF.4 does NOT rewrite any prior T.11 / S1.3 / T.12.x /
    FF.1 / FF.2 / FF.3 hash.
  - FF.4 does NOT mutate SEED.len() (stays at 54).
  - FF.4 does NOT promote any open proposal to Accepted.
  - FF.4 does NOT change S1.3a / FF.2 / FF.3 court decisions.
  - FF.4 does NOT generate CUDA kernels.
  - FF.4 does NOT decide contraindications or challenges.
  - FF.4 does NOT mutate the registry crate.

----------------------------------------------------------------
Next campaign (panel-authorized post-FF.4)
----------------------------------------------------------------

FF.5 ProposalSchemaUpgradePolicy. Defines how proposal schema
upgrades may re-render historical proposal artifacts without
erasing the old artifact hashes or confusing the court lineage.
Core rule: schema upgrade != silent artifact rewrite. After
FF.5: S1.3d budget pruning + redundancy suppression begins.
