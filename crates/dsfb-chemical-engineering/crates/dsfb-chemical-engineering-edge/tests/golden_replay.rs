//! Frozen golden pipeline-replay-hash gate.
//!
//! `verify-replay` runs each synthetic dataset *twice in one process* and checks the two hashes agree
//! — that proves the hasher is stable, but it cannot prove the pipeline reproduces a *reference* result
//! across machines or compiler versions. This test pins the canonical `replay_hash` of every synthetic
//! dataset to a frozen constant, so any drift — codegen/LLVM change, accidental non-determinism, or an
//! intended change to the grammar/fusion semantics — fails loudly. It is the cross-run analogue of the
//! frozen `atlas_hash_v1` / `corpus_hash_v1` gates. If a change is intentional, re-freeze the hex here.

use dsfb_chemical_engineering_edge::pipeline::PipelineConfig;
use dsfb_chemical_engineering_edge::{cli, pipeline};

/// (dataset name, frozen canonical replay hash). Minted on the pinned toolchain (see
/// `rust-toolchain.toml`); re-freeze deliberately if the grammar/fusion semantics change.
// GOVERNED RE-FREEZE (Phase-C executable, 2026-05-26): the executed detector bank grew 14 → 18 (mewma,
// dpca, mosum, mmd appended to `build_bank`), adding four new per-detector timelines to the fused-grammar
// stream, so every dataset's canonical replay hash shifts. Re-minted from the deterministic run
// (verify-replay 6/6 self-consistent run-to-run before re-freezing).
const GOLDEN: &[(&str, &str)] = &[
    (
        "synth_step_drift",
        "ca0bfbf3899c23676537fe7bad84625cd881d204c3995bd20b7501f1d8da0b03",
    ),
    (
        "synth_ramp_drift",
        "6e6d6aafbfb32417103a94bacb6c15919e3a8dc5a5cc2404aac9de5a860f7fa0",
    ),
    (
        "synth_sensor_bias",
        "1a7bb7c607bf9ddd447d7d50d0dc5d7bd4ba5ebc5c5fca8c430f3f9a6d97c225",
    ),
    (
        "synth_oscillation",
        "d5f26712005fe98b96a0f1a897dfa657f242b39eeedbbd2c51cbd9548a024782",
    ),
    (
        "synth_wide_step",
        "5f0fab5e432d0628421f6165c73b1d7a4c0b8438713277e3abd60ddd36b4e790",
    ),
    (
        "synth_wide_ramp",
        "237a04bf718927c79c0cc72e26f29f219ab0e8c922abb8209c8c287f3eabc6cc",
    ),
];

#[test]
fn synthetic_replay_hashes_match_frozen_golden() {
    let cfg = PipelineConfig::default();
    let suite = cli::synthetic_suite();
    assert_eq!(
        suite.len(),
        GOLDEN.len(),
        "golden set must cover the whole synthetic suite"
    );
    for d in &suite {
        let want = GOLDEN
            .iter()
            .find(|(n, _)| *n == d.name)
            .map(|(_, h)| *h)
            .unwrap_or_else(|| panic!("no golden hash pinned for {}", d.name));
        let res = pipeline::analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
        assert_eq!(
            res.replay_hash, want,
            "replay-hash drift for {} — a determinism regression, OR an intended grammar/fusion change \
             (if intended, re-freeze the hex in GOLDEN)",
            d.name
        );
    }
}
