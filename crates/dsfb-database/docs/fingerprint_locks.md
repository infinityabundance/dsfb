# Fingerprint Locks

Four SHA-256-pinned tests guard the paper's claims against silent drift.

## `paper_fingerprint_is_pinned`

- Location: [tests/deterministic_replay.rs](../tests/deterministic_replay.rs)
- Hashes: the full residual stream (all five classes concatenated in
  deterministic order) produced by `reproduce --seed 42` on the TPC-DS
  tier.
- What breaks it: any change to adapter parsing, residual construction,
  numeric rounding, or sample ordering.
- What does not break it: display-layer changes (plots.rs), metadata in
  report JSON, CLI flag renames.

## `paper_episode_fingerprint_is_pinned`

- Location: [tests/deterministic_replay.rs](../tests/deterministic_replay.rs)
- Hashes: the full episode stream produced by the motif engine on the
  TPC-DS residuals.
- What breaks it: any change to motif state machine transitions,
  thresholds, or minimum-dwell values.
- What does not break it: episode *presentation* in the report layer.

## `paper_ceb_episode_fingerprint_is_pinned`

- Location: [tests/deterministic_replay.rs](../tests/deterministic_replay.rs)
- Hashes: the episode stream for the bundled CEB sample.
- Separate from TPC-DS because CEB is real-world input; its residual
  construction path is fingerprinted independently to catch adapter-only
  regressions.

## `non_claim_block_is_verbatim`

- Location: [tests/non_claim_lock.rs](../tests/non_claim_lock.rs)
- Hashes: the string array in [src/non_claims.rs](../src/non_claims.rs).
- What breaks it: any edit to the non-claim text.
- Why verbatim: the paper's honesty contract requires that the claims the
  crate does *not* make remain stable between drafts.

## What is not fingerprinted

Deliberately excluded from the locks:

- PNG byte contents. Figure layout is free to iterate.
- Report text (`reproduce.txt`, `run.txt` stdout). Human-oriented.
- Log output. Non-deterministic timing.
- `out/*.csv` contents (episode CSV is derived from the episode stream,
  which is already locked; lock duplication would create false positives).
