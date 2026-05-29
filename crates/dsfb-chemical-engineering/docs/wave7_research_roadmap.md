# Wave-7 research roadmap — catalogued / handoff (not executed in-repo)

> **Status: catalogued / handoff.** The Wave-7 items below need external tooling, live infrastructure, or a
> dedicated build target that is out of scope for the deterministic in-repo test loop. They are disclosed here
> as prior-art roadmap (the standing breadth strategy) and, where useful, anchored to the in-repo objects that
> already partially realise them. Listing an item is an **enabling disclosure, not a capability claim** —
> nothing below is run end-to-end in this repo. The items already *built* in Wave 7 (interval physics-informed
> envelopes, multi-physics cross-witnessing, multi-scale fusion, spectral grammar, Merkle-DAG amendments,
> uncertainty dashboard, signature discovery, DSFB-Bench, safety dossier, proof-obligation ledger) live in the
> edge crate and are not repeated here.

## Curated AVOIDs (do not implement)
- **#10 post-quantum / quantum-resistant hashing** — out of scope by the maintainer's decision; SHA-256 +
  hash-chaining is the sealing primitive throughout.
- **#17 a *generic cross-domain* DSFB-Core extraction** — keep DSFB chemical-focused; do **not** genericise the
  framework for non-chemical domains. (The chemical `no_std` embedded core `dsfb-chemical-engineering-core` is
  **kept** — it is the chemical embedded profile, not the genericisation #17 warns against.)

## Formal methods (Lean 4 — DONE + verified; Coq — handoff)
- **Lean 4-verified grammar + fusion theorems — DONE.** `formal/lean/DsfbGrammar.lean` (pure Lean 4 core, no
  Mathlib; `lake build` verifies) machine-proves, over unbounded `Int`: grammar totality, valid≠SensorFault,
  beyond-bound≠nominal (the three Kani obligations, now unbounded), the compound rule, **quorum soundness**, and
  **episode-compression monotonicity** — i.e. two of the three previously-open obligations are now proven.
  `proof_obligations::ProofObligationLedgerV1` records this (`Lean4Verified`; 5 machine-checked / 1 open).
- **Replay determinism** remains the one open obligation — gated empirically by `verify-replay` 6/6 + the
  golden hashes; a Lean/Coq proof of it is future work.
- **Coq / Rocq port — DONE + verified.** `formal/coq/DsfbGrammar.v` (Rocq Prover 9.1.1, needs `rocq-stdlib`)
  cross-checks the same theorems in a second prover kernel, with `classifyAxis` additionally modelled over `Z`.
  `coqc DsfbGrammar.v` verifies. So the grammar/fusion obligations are now machine-checked by **three**
  independent tools (Kani, Lean 4, Coq); only replay determinism remains empirical.

## Compute / backends (handoff — needs GPU + build targets)
- **Unified heterogeneous backend** (one executor over CUDA + CPU + edge + WebGPU/WGSL; auto-partition
  GPU-seal vs edge-grammar). Anchored: the CPU/CUDA backends already produce a byte-identical `evidence_root`
  (the cross-backend invariant); a WGSL backend must join that digest-equivalence gate. Large; needs a WebGPU
  target + device.
- **Adaptive kernel auto-tuning** that preserves the byte-exact `evidence_root` (Nsight-driven selection gated
  by `DigestEquivalenceHarnessV1`). Perf tuning is via the **Nsight handoff** (the user runs `ncu`/`nsys` and
  pastes back — see `crates/dsfb-chemical-engineering-cuda/reports/CPU_VS_GPU_HANDOFF.md`); a tuning change is
  accepted only if the evidence root is unchanged. *(AVOID #10.)*

## Live ingestion / interactivity (WASM court simulator DONE; live streaming still handoff)
- **OPC-UA / MQTT / Sparkplug historian ingestion + live streaming Court Record emission.** Needs a live
  broker/PLC and safe async bindings; the deterministic batch path (`historian` / `data-readiness`) is the
  in-repo, reproducible analogue. Streaming would emit incremental sealed records on the same grammar.
- **Interactive Chemical Court simulator** (Rust core → WASM; replay a residual stream under a sandboxed
  "what-if" *admissibility envelope* over *immutable* evidence — a HAZOP/training tool). **DONE** —
  `crates/dsfb-chemical-engineering-wasm` (standalone, raw `extern "C"` exports, no wasm-bindgen) builds the
  module from the dependency-free `no_std` `dsfb-chemical-engineering-core` and ships a static HTML/JS shell
  (`web/`): the operator drags the envelope `k` / grazing band / drift window and watches the same residual
  stream re-classify, while the residual SHA-256 (shown) stays constant — amendments never touch the sealed
  record. Pure logic is host-tested (`cargo test`); the chemical sample is a labelled synthetic residual (not
  plant data). The immutable-evidence + append-only-amendment semantics for *full Court Records* remain
  enforced by `redaction::TamperEvidenceSealV1` + `amendment_dag::MerkleDagAmendmentChainV1`.

## Scale / ecosystem (handoff — needs infra / is large)
- **Dynamic topology & residence discovery** (rank-based, human-veto, evidence-graded — extends `topology.rs`).
  Tractable in-repo later; deferred as a focused pass (it touches the topology authority).
- **Global open atlas & evidence registry** (Git + content-addressed pinning + SHA-256 authority chain for
  community detectors/heuristics/anonymised Court Records). Needs hosting/infra + governance; **publishing is
  USER-ONLY**. The content-addressing + authority-chain primitives exist (`atlas_hash_v1`, the tamper seal).
  *(AVOID #17 — keep the registry chemical-scoped.)*
- **Densor IR** (a small documented intermediate representation for any residual stream + a compiler from
  detector outputs to the DSFB grammar). Design-stage; an in-repo follow-up could specify the IR without the
  genericisation #17 warns against (chemical residual streams only).
- **Synthetic fault & historian generator** (physics + reproducible-stochastic; extends `gen_instrumented.py`).
  Tractable in-repo later; the existing instrumented-dataset generator is the seed.

## Honesty
Everything here is **catalogued / handoff, not executed in-repo**. The framework's value claim is unchanged: it
does not assert these are realised today; it discloses, as prior art, the apparatus designed to absorb them and
the exact in-repo objects each builds on. The two AVOIDs are honoured.
