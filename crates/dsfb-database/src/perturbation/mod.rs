//! Perturbation harness.
//!
//! Real SQL workloads lack ground-truth labels for "what went wrong when."
//! The honest replacement (per Strategy A in the panel discussion) is
//! controlled perturbation injection: take a clean trace, deterministically
//! inject a known fault inside a known time window, run DSFB-Database, and
//! check whether the emitted episodes overlap that window.
//!
//! Each perturbation:
//!   * has a name + class
//!   * is restricted to a `[t_start, t_end]` window (the *ground-truth window*)
//!   * is deterministic given a seed
//!   * is documented in `spec/perturbations.yaml`
//!
//! The five perturbations cover the five motif classes one-to-one so the
//! evaluation cleanly maps motif → injection → window → F1.

use crate::residual::{
    cache_io, cardinality, contention, plan_regression, workload_phase, ResidualStream,
};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerturbationClass {
    LatencyInjection,
    StatisticsStaleness,
    LockHold,
    CacheEviction,
    WorkloadShift,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerturbationWindow {
    pub class: PerturbationClass,
    pub t_start: f64,
    pub t_end: f64,
    pub channel: String,
    pub magnitude: f64,
    pub seed: u64,
}

/// Build a TPC-DS-shaped trace with all five perturbations injected at
/// disjoint, documented windows. The returned (stream, ground-truth windows)
/// pair is the empirical evidence for §8 of the paper.
pub fn tpcds_with_perturbations(seed: u64) -> (ResidualStream, Vec<PerturbationWindow>) {
    tpcds_with_perturbations_scaled(seed, 1.0)
}

/// Same harness, but with each perturbation's *magnitude* multiplied by
/// `scale`. `scale = 1.0` reproduces the canonical pinned-fingerprint
/// stream exactly (same RNG draw sequence, same byte output). Lower
/// scales produce subthreshold perturbations — the residual is still
/// present but barely above noise — and the stress sweep
/// (`stress-sweep` subcommand) reports per-motif F1 across a range
/// of scales so we can see *where each motif breaks down*, not just
/// that it works at the published baseline.
pub fn tpcds_with_perturbations_scaled(
    seed: u64,
    scale: f64,
) -> (ResidualStream, Vec<PerturbationWindow>) {
    let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
    let mut stream = ResidualStream::new(if (scale - 1.0).abs() < 1e-12 {
        format!("tpcds-perturbed-seed{seed}")
    } else {
        format!("tpcds-perturbed-seed{seed}-scale{:.3}", scale)
    });
    let mut windows = Vec::new();
    let bucket_seconds = 30.0;

    // Stable backbone — 30 minutes (1800 s) of clean traffic across q1..q99.
    // The RNG draw sequence here is identical to the original
    // `tpcds_with_perturbations` so that scale=1.0 reproduces the
    // canonical fingerprint byte-for-byte.
    for t_int in 0..1800 {
        let t = t_int as f64;
        let q = (t_int % 99) + 1;
        let qid = format!("q{}", q);
        let true_rows: f64 = 5000.0 * (1.0 + rng.gen_range(0.0..0.4));
        let est_rows = true_rows * (1.0 + rng.gen_range(-0.08..0.08));
        cardinality::push(&mut stream, t, &qid, est_rows, true_rows);
        plan_regression::push_latency(&mut stream, t, &qid, 50.0 + rng.gen_range(-2.0..2.0), 50.0);
        cache_io::push_hit_ratio(&mut stream, t, "tpcds", 0.95, 0.95 + rng.gen_range(-0.005..0.005));
    }

    // 1) Latency injection — at scale=1.0 q42 latency runs at 6× baseline.
    let win = PerturbationWindow {
        class: PerturbationClass::LatencyInjection,
        t_start: 200.0,
        t_end: 280.0,
        channel: "q42".into(),
        magnitude: 1.0 + 5.0 * scale,
        seed,
    };
    for t_int in 200..280 {
        let t = t_int as f64;
        // 50 ms baseline + 250 ms*scale extra latency + ±10 ms jitter
        plan_regression::push_latency(
            &mut stream,
            t,
            "q42",
            50.0 + 250.0 * scale + rng.gen_range(-10.0..10.0),
            50.0,
        );
    }
    windows.push(win);

    // 2) Statistics staleness — at scale=1.0 q17 cardinality est/actual = 30×.
    let win = PerturbationWindow {
        class: PerturbationClass::StatisticsStaleness,
        t_start: 600.0,
        t_end: 720.0,
        channel: "q17".into(),
        magnitude: 1.0 + 29.0 * scale,
        seed,
    };
    for t_int in 600..720 {
        let t = t_int as f64;
        let true_rows: f64 = 30000.0;
        let est_rows = true_rows / (1.0 + 29.0 * scale);
        cardinality::push(&mut stream, t, "q17", est_rows, true_rows);
    }
    windows.push(win);

    // 3) Lock hold — wait + chain-depth ramp. At scale=1.0, max wait 1.55 s.
    let win = PerturbationWindow {
        class: PerturbationClass::LockHold,
        t_start: 900.0,
        t_end: 1020.0,
        channel: "row_lock".into(),
        magnitude: 1.5 * scale,
        seed,
    };
    for t_int in 900..1020 {
        let t = t_int as f64;
        let progress = (t - 900.0) / 120.0;
        let wait_s = 0.05 + 1.5 * scale * progress;
        contention::push_wait(&mut stream, t, "row_lock", wait_s);
        let depth = (1.0 + 4.0 * scale * progress) as usize;
        contention::push_chain_depth(&mut stream, t, "row_lock", depth);
    }
    windows.push(win);

    // 4) Cache eviction — at scale=1.0 hit_ratio drops 0.95 → 0.50.
    let win = PerturbationWindow {
        class: PerturbationClass::CacheEviction,
        t_start: 1200.0,
        t_end: 1320.0,
        channel: "tpcds".into(),
        magnitude: 0.45 * scale,
        seed,
    };
    for t_int in 1200..1320 {
        let t = t_int as f64;
        cache_io::push_hit_ratio(
            &mut stream,
            t,
            "tpcds",
            0.95,
            0.95 - 0.45 * scale + rng.gen_range(-0.02..0.02),
        );
    }
    windows.push(win);

    // 5) Workload shift — at scale=1.0 JSD elevates to ~0.45.
    let win = PerturbationWindow {
        class: PerturbationClass::WorkloadShift,
        t_start: 1500.0,
        t_end: 1680.0,
        channel: "tpcds".into(),
        magnitude: 0.4 * scale,
        seed,
    };
    let mut t = 1500.0;
    while t < 1680.0 {
        let d = 0.05 + 0.4 * scale + rng.gen_range(-0.05..0.05);
        workload_phase::push_jsd(&mut stream, t, "tpcds", d);
        t += bucket_seconds;
    }
    windows.push(win);

    stream.sort();
    (stream, windows)
}
