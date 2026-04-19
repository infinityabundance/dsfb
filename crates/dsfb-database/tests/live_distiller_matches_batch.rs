//! Batch ↔ live plan-regression parity lock.
//!
//! The batch CSV path in [`dsfb_database::adapters::postgres`] and the
//! live distiller in [`dsfb_database::live::distiller`] both consume
//! `pg_stat_statements` counter snapshots. They differ only in the
//! transport (CSV rows vs live rows) and in the workload-phase
//! normalisation (batch uses global max, live uses running max —
//! documented honestly in both docs and paper §Live). On the
//! `PlanRegression` channel they must agree byte-for-byte: the same
//! counter triples fed through each path must produce the same
//! `(t, value, channel)` triples.
//!
//! This test is the lock for the shared `PgssQidState::push_snapshot`
//! invariant: if the live and batch paths ever disagree on the per-
//! call mean or the baseline, this test fires before any field report
//! can. It is the only test that binds the batch adapter's arithmetic
//! to the live adapter's arithmetic — do not delete.

#![cfg(feature = "live-postgres")]

use dsfb_database::adapters::postgres::load_pg_stat_statements;
use dsfb_database::live::distiller::{DistillerState, PgssRow, Snapshot};
use dsfb_database::residual::{ResidualClass, ResidualSample};
use std::io::Write;
use tempfile::tempdir;

/// A deterministic synthetic counter trajectory: two qids, six
/// snapshots, a plan-regression onset in qid `qA` at snap 4.
fn synthetic_trajectory() -> Vec<(f64, Vec<(&'static str, u64, f64)>)> {
    // (snapshot_t, [(qid, calls, total_exec_ms), ...])
    // Baseline: qA 10 ms/call, qB 5 ms/call.
    // At snap 4, qA's per-call mean jumps to 30 ms (plan regression).
    vec![
        (0.0, vec![("qA", 0, 0.0), ("qB", 0, 0.0)]),
        (1.0, vec![("qA", 100, 1000.0), ("qB", 200, 1000.0)]),
        (2.0, vec![("qA", 200, 2000.0), ("qB", 400, 2000.0)]),
        (3.0, vec![("qA", 300, 3000.0), ("qB", 600, 3000.0)]),
        (4.0, vec![("qA", 400, 4000.0), ("qB", 800, 4000.0)]),
        (5.0, vec![("qA", 500, 7000.0), ("qB", 1000, 5000.0)]),
        (6.0, vec![("qA", 600, 10000.0), ("qB", 1200, 6000.0)]),
    ]
}

fn write_csv(traj: &[(f64, Vec<(&'static str, u64, f64)>)]) -> std::path::PathBuf {
    let dir = tempdir().unwrap();
    let p = dir.path().join("pgss.csv");
    let mut f = std::fs::File::create(&p).unwrap();
    writeln!(f, "snapshot_t,query_id,calls,total_exec_time_ms").unwrap();
    for (t, rows) in traj.iter() {
        for (qid, calls, total) in rows.iter() {
            writeln!(f, "{},{},{},{}", t, qid, calls, total).unwrap();
        }
    }
    // Leak the tempdir so the file survives for the duration of the
    // test; rely on the OS to reap /tmp entries on next boot.
    let _leaked = dir.keep();
    p
}

fn plan_regression_triples(samples: &[ResidualSample]) -> Vec<(u64, String, u64)> {
    // Encode (t_ms, channel, value_bits) as a byte-comparable triple.
    // Using to_bits() captures floating-point byte equality exactly;
    // multiplying t by 1000 and rounding captures the timestamp with
    // ms precision (both paths emit t in whole seconds here).
    let mut out: Vec<(u64, String, u64)> = samples
        .iter()
        .filter(|s| s.class == ResidualClass::PlanRegression)
        .map(|s| {
            (
                (s.t * 1000.0) as u64,
                s.channel.clone().unwrap_or_default(),
                s.value.to_bits(),
            )
        })
        .collect();
    out.sort();
    out
}

#[test]
fn plan_regression_residuals_are_byte_equal_between_paths() {
    let traj = synthetic_trajectory();

    // Batch path
    let csv_path = write_csv(&traj);
    let batch_stream = load_pg_stat_statements(&csv_path).expect("batch load");
    let batch_triples = plan_regression_triples(&batch_stream.samples);

    // Live path
    let mut d = DistillerState::new();
    let mut live_samples: Vec<ResidualSample> = Vec::new();
    for (t, rows) in traj.iter() {
        let pgss: Vec<PgssRow> = rows
            .iter()
            .map(|(qid, calls, total)| PgssRow {
                query_id: qid.to_string(),
                calls: *calls,
                total_exec_time_ms: *total,
            })
            .collect();
        let snap = Snapshot {
            t: *t,
            pgss,
            ..Default::default()
        };
        live_samples.extend(d.ingest(&snap));
    }
    let live_triples = plan_regression_triples(&live_samples);

    assert_eq!(
        batch_triples, live_triples,
        "plan_regression residuals drifted between batch CSV path and live distiller path"
    );
    assert!(
        !batch_triples.is_empty(),
        "sanity: the synthetic trajectory must produce at least one plan_regression residual"
    );
}
