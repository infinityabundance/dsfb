# Molecular-corpus companion — `dsfb-chemical-engineering-molecular-corpus` (design note)

**Status: design only (P56). NOT built, NOT part of this artifact, NOT a dependency of any shipped
crate, and it does NOT mutate the soft-sensor corpus.** This note exists so the boundary is explicit and
the future direction is disclosed as prior art. The shipped `dsfb-chemical-engineering-corpus` is, and
remains, a **soft-sensor dataset authority catalogue** (cheap sensors → hard-to-measure target). PubChem-
scale *molecular* densors are a separate, future companion corpus described here.

## 1. Why a separate crate (the boundary)

The soft-sensor corpus and a molecular corpus share the **DSFB/Densor posture** — deterministic
inference from residuals + structure, no probability model — but nothing else: different data
(time-series process residuals vs static molecular descriptors), different provenance (UCI/Kaggle/Zenodo
process datasets vs PubChem/ChEMBL compound records), different authority hash, different validation
gates. Folding molecules into the soft-sensor corpus would blur a clean catalogue and silently change
`corpus_hash_v1`. So the companion is a **new crate**, with its own frozen `molecular_corpus_hash_v1`,
mirroring the existing discipline: `no_std`, `&'static` records, a mandatory provenance `SourceRef` per
entry, **no compound bytes vendored**, deterministic validation, and a hash-sealed catalogue.

## 2. What it would catalogue (design)

- **`CompoundDensorV1`** — a per-compound record: identifiers (InChIKey, PubChem CID), a small set of
  **cheap, ubiquitous molecular descriptors** (MW, logP, TPSA, H-bond donors/acceptors, rotatable bonds,
  ring count) as the "cheap sensors", and a hard-to-measure **target** (e.g. solubility, permeability,
  a bioassay outcome) — the exact soft-sensor framing, lifted to chemistry: cheap descriptors infer an
  expensive measurement.
- **PubChem/ChEMBL shards** — provenance-bound references (CID ranges, assay AIDs, dataset DOIs) with URL
  + licence + access flag, classified on the **same four P53 tiers** (licence/access confidence,
  redistribution policy, source-authority kind). No compound bytes vendored; shards are *pointers*.
- **Descriptor admissibility envelopes** — the molecular analogue of the process admissibility envelope:
  a deterministic, regime-conditioned bound on each descriptor (e.g. Lipinski-style ranges as a
  *structural* envelope, not a probabilistic filter), against which a compound's descriptor residual is
  read by the same drift/slew/envelope grammar.
- **Fingerprint motifs** — deterministic structural motifs (e.g. ECFP/MACCS bit signatures) catalogued
  as `&'static` motif records, the molecular counterpart of the process-fault signature bank.
- **Scaffold + confuser dockets** — per-scaffold groupings and an explicit **confuser docket** (compounds
  whose cheap descriptors collide but whose targets differ), mirroring the atlas's confuser discipline so
  the limits of descriptor-only inference are disclosed, not hidden.
- **H7–H16 molecular heuristics** — extending the process heuristics bank (H1–H6) with molecular
  heuristics (descriptor-envelope violations, scaffold-class drift, fingerprint-motif recruitment),
  each a curated `&'static` record with the same "advisory candidate, never root cause" bound.

## 3. What it preserves (the discipline carries over verbatim)

- **Determinism + replay**: descriptors quantised to fixed point; the same drift/slew/envelope grammar;
  a `CompoundCourtRecord` analogous to the Chemical Court Record, with a `molecular_evidence_root`.
- **Authority separation**: the molecular corpus is *authority* (what a descriptor residual is allowed to
  mean), separate from execution — exactly the atlas/corpus vs edge/cuda split.
- **No overclaim**: cataloguing a compound asserts no predictive-accuracy claim, no bioactivity claim, no
  tox/hazard claim; `implementation_status` honestly marks `Executed` vs `Catalogued`. No compound bytes
  redistributed; the prior-art is the **deterministic descriptor-residual inference technology**, not the
  public chemical data.

## 4. Honest bounds

- This is a **design note**, not a crate. None of it is implemented; the soft-sensor corpus is unchanged
  and remains the sole corpus in the artifact.
- Descriptor-only inference is **fundamentally limited** (the confuser docket exists precisely to disclose
  that cheap descriptors cannot resolve activity cliffs); the companion would target deterministic
  *structural screening + envelope witnesses*, not property prediction, and would say so.
- Building it is a separate effort with its own provenance/licence review of PubChem/ChEMBL terms (which
  is exactly why it is a separate, separately-hashed crate, classified on the P53 tiers from day one).
