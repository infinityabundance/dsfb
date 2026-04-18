# Evaluation Model

`dsfb-database` is evaluated in two tiers. The tiers ask different questions
and carry different evidence weight.

## Tier 1 — Controlled-perturbation (TPC-DS)

**Question asked:** given a deterministic trace with known injected
perturbation windows, does the motif grammar recover them?

**Why this tier exists:** ground truth is available. Precision, recall, and
F1 are meaningful numbers.

**What counts as a claim:** TPC-DS detection F1 under the pinned seed and
pinned thresholds. Any change to grammar thresholds requires re-running
`reproduce` and updating the paper's Table 1.

**What does not count as a claim:** real-world gray-failure detection rates.
The TPC-DS trace is a deterministic fiction. It cannot certify behaviour on
production workloads.

## Tier 2 — Real-world observation (Snowset, SQLShare-text, CEB, JOB)

**Question asked:** when we run the same residual-observation pipeline on
publicly available real SQL traces, what does the emitted episode structure
look like?

**Why this tier exists:** to demonstrate that the pipeline produces
operator-legible output on data the authors did not generate.

**What counts as a claim:** structural observations — episode counts,
channel distributions, saturation behaviour — each individually named in
the paper and each traceable to a PNG or CSV artefact.

**What does not count as a claim:** detection F1. There is no ground truth
in Snowset, SQLShare, CEB, or JOB for the residual classes this crate
constructs.

## Honesty Boundaries

1. Every figure caption in `paper/dsfb-database.tex` that corresponds to a
   Tier-2 dataset carries an explicit "no ground truth; structural
   observation only" disclaimer.
2. The abstract splits Tier-1 (F1 number) and Tier-2 (structural
   observations) into separate sentences.
3. The non-claim block (`src/non_claims.rs`) enumerates, verbatim, what
   the crate does *not* infer. The block is compile-time locked by
   `tests/non_claim_lock.rs`.

## Reproducibility

```bash
# Tier 1
./target/release/dsfb-database reproduce --seed 42 --out out

# Tier 2 (operator supplies the files)
./target/release/dsfb-database run --dataset snowset --path <file>
./target/release/dsfb-database run --dataset sqlshare-text --path <file>
./target/release/dsfb-database run --dataset ceb --path <file>
./target/release/dsfb-database run --dataset job --path <file>
```

The Tier-1 command produces the pinned fingerprint artefacts;
`cargo test --release` verifies them.
