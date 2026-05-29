# hax — Rust → F\* extraction (INSTALLED + RUN; real F\* model generated)

[hax](https://github.com/cryspen/hax) (Cryspen) translates a safe subset of Rust into the input languages of proof
assistants (**F\***, **Coq/Rocq**, …) so the proven object is extracted *from the real Rust source* rather than
hand-ported — closing the drift gap against the hand-written `formal/coq/DsfbGrammar.v`.

## What was actually done (2026-05-27) — installed + ran, real artifact
Full hax was installed in this sandbox (frontend + driver + OCaml engine):
```fish
cargo +nightly install --git https://github.com/cryspen/hax cargo-hax   # frontend
git clone https://github.com/cryspen/hax; and cd hax; and ./setup.sh    # driver-hax-frontend-exporter + hax-engine 0.3.7 (opam)
cd crates/dsfb-chemical-engineering-core; and cargo hax into fstar       # extract
```
Installed: `cargo-hax` + `driver-hax-frontend-exporter` + `hax-engine 0.3.7`.

**Result: hax extracted the `no_std` core grammar to a 716-line F\* model** —
[`Dsfb_chemical_engineering_core.fst`](Dsfb_chemical_engineering_core.fst) (a verbatim copy of
`crates/dsfb-chemical-engineering-core/proofs/fstar/extraction/…`; run log in
[`run_fstar_extraction.txt`](run_fstar_extraction.txt)). It contains the real grammar, generated from the Rust:
- `let v_SCALE: i64 = mk_i64 1000000`
- `type t_CoordClass = | CoordClass_Interior | CoordClass_Grazing | CoordClass_Outside`
- `let classify_axis (v lo hi band_scaled: i64) : t_CoordClass = ...` — the actual `i128`-promoted axis classifier
- `type t_FixedTriple`, `type t_FixedEnvelope`, `let impl_FixedEnvelope__symmetric (k_scaled band_scaled: i64)`,
  and the grammar-state types.

This is a genuine extraction — the F\* object provably *is* the Rust that runs (no hand-port). It is the strongest
form of the "model mirrors the code" claim, now tool-generated rather than a reviewer's trust assumption.

## Honest scope of this result
hax produced a faithful **model**; it is not yet a **proof**. The next step (future work, not a blocker) is to state
and discharge lemmas over the extracted `classify_axis` / `eval` / `classify` in F\* (e.g. totality, the exhaustive
`Interior/Grazing/Outside` partition) — the same obligations Kani proves bounded and `formal/coq/DsfbGrammar.v`
proves by hand, now on a model extracted from the source. ProVerif/SSProve (protocol/crypto) are out of scope.

## What it does NOT certify
The extraction is only as strong as the lemmas later written against it. Floating-point and the `std`/CUDA paths are
out of scope; this targets the `no_std` integer grammar (the embedded sibling of the float pipeline).
