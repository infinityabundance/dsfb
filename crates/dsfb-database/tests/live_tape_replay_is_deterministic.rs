//! Tape → episodes byte-determinism lock.
//!
//! The seventh non-claim states the boundary: engine → tape is not
//! reproducible; tape → episodes is. This test writes a synthetic
//! tape from a pinned seed, replays it twice through the batch
//! motif engine, and asserts the episode fingerprints are byte-equal.
//! Any drift here means the replay path has lost determinism —
//! either a HashMap iteration leak, a floating-point ordering
//! dependency, or a hidden system-clock read — and the lock for the
//! live adapter's determinism claim is gone.

#![cfg(feature = "live-postgres")]

use dsfb_database::grammar::{replay, MotifEngine, MotifGrammar};
use dsfb_database::live::tape::{load_and_verify, Tape};
use dsfb_database::residual::{plan_regression, workload_phase, ResidualStream};
use tempfile::tempdir;

fn build_synthetic_stream(seed: u64) -> ResidualStream {
    // Deterministic pseudo-random stream seeded by `seed`; the
    // sequence is fully reproducible so two calls with the same
    // seed return the byte-identical stream.
    use rand::{Rng, SeedableRng};
    let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
    let mut s = ResidualStream::new("synthetic-live-tape");
    // 60 seconds of 1 Hz samples: steady baseline, then a
    // plan-regression onset around t=20, plus a workload_phase
    // spike around t=40.
    for t in 0..60u64 {
        let t = t as f64;
        let baseline = 10.0;
        let latency = if t >= 20.0 && t <= 35.0 {
            baseline * (1.0 + 2.0 * rng.gen::<f64>())
        } else {
            baseline + rng.gen::<f64>() * 0.1
        };
        plan_regression::push_latency(&mut s, t, "qA", latency, baseline);
        let jsd = if t >= 40.0 && t <= 55.0 {
            0.5 + rng.gen::<f64>() * 0.2
        } else {
            rng.gen::<f64>() * 0.05
        };
        workload_phase::push_jsd(&mut s, t, "bucket", jsd);
    }
    s.sort();
    s
}

fn replay_episodes_fingerprint(stream: &ResidualStream) -> String {
    let engine = MotifEngine::new(MotifGrammar::default());
    let episodes = engine.run(stream);
    replay::fingerprint_hex(&episodes)
}

#[test]
fn tape_replay_is_byte_deterministic_for_same_seed() {
    let dir = tempdir().unwrap();
    let tape_path = dir.path().join("t.jsonl");
    // Build the synthetic stream and write it to a tape.
    let stream = build_synthetic_stream(42);
    let mut tape = Tape::create(&tape_path, "synthetic:seed=42").unwrap();
    tape.append(&stream.samples).unwrap();
    let manifest = tape.finalize().unwrap();
    // Two independent replays must produce byte-identical episode
    // fingerprints.
    let (s1, _) = load_and_verify(&tape_path).unwrap();
    let (s2, _) = load_and_verify(&tape_path).unwrap();
    // Two loads of the same tape must fingerprint-equal each other.
    // The `source` label differs from the in-memory stream (the
    // tape encodes the label supplied to `Tape::create`, not the
    // original `ResidualStream::new` label) so we don't compare
    // against `stream.fingerprint()` — we compare replay outputs.
    assert_eq!(s1.fingerprint(), s2.fingerprint());
    let fp1 = replay_episodes_fingerprint(&s1);
    let fp2 = replay_episodes_fingerprint(&s2);
    assert_eq!(fp1, fp2, "tape replay fingerprint drifted across invocations");
    // A second tape written from the same seed must have the same
    // SHA-256 as the first (the ResidualStream's sample order is
    // deterministic, and the JSONL encoding is deterministic).
    let tape_path_2 = dir.path().join("t2.jsonl");
    let stream_2 = build_synthetic_stream(42);
    let mut tape_2 = Tape::create(&tape_path_2, "synthetic:seed=42").unwrap();
    tape_2.append(&stream_2.samples).unwrap();
    let manifest_2 = tape_2.finalize().unwrap();
    assert_eq!(
        manifest.sha256, manifest_2.sha256,
        "tape bytes drifted across invocations with identical seed",
    );
}

#[test]
fn tape_sample_count_survives_roundtrip() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("t.jsonl");
    let stream = build_synthetic_stream(7);
    let mut tape = Tape::create(&p, "synthetic:seed=7").unwrap();
    tape.append(&stream.samples).unwrap();
    let manifest = tape.finalize().unwrap();
    assert_eq!(manifest.sample_count as usize, stream.samples.len());
    let (loaded, _) = load_and_verify(&p).unwrap();
    assert_eq!(loaded.samples.len(), stream.samples.len());
}
