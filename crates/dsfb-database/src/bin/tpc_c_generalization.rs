#![forbid(unsafe_code)]

//! Phase-B2: TPC-C generalization replay through the `pg_stat_statements`
//! adapter.
//!
//! The five published adapters (Snowset, SQLShare, CEB, JOB, TPC-DS) are
//! the corpora the motif envelope was tuned against. A fair
//! generalization question is: **does the published envelope produce
//! sensible output on a workload shape we have not used for tuning?**
//! This binary answers that by synthesising a TPC-C-style
//! `pg_stat_statements` snapshot CSV — the canonical OLTP benchmark, with
//! a call-distribution and latency profile that bears no relation to any
//! of the five analytics corpora — and feeding it through the *unchanged*
//! [`dsfb_database::adapters::postgres::load_pg_stat_statements`] adapter.
//!
//! The synthesised workload plants two ground-truth perturbations the
//! adapter can in principle see (plan-regression and workload-phase; the
//! adapter does not emit cardinality / contention / cache residuals):
//!
//!   * **Plan regression**: the highest-share query (NewOrder hot-path
//!     SELECT) has its mean latency tripled from snapshot 60 onward
//!     (t = 3600 s).
//!   * **Workload-phase shift**: the call mix concentrates on three
//!     queries from snapshot 80 onward (t = 4800 s), which drops the
//!     per-snapshot call-share entropy.
//!
//! The binary records, per motif, whether the engine opened at least one
//! episode whose `[t_start, t_end]` overlaps the planted ground-truth
//! window. Artefacts land at `<out>/tpc_c_generalization.csv`, outside
//! every fingerprint-locked path.
//!
//! Generalization claim: the default [`MotifParams`] envelope was never
//! fitted against a TPC-C-shaped stream. If it still opens episodes
//! over-lapping both planted windows, the envelope is not over-fit to the
//! five-corpus tuning set. If it does not, we report that honestly in
//! `out/tpc_c_generalization.csv` — non-claim #5 is unchanged either way.

use anyhow::Result;
use clap::Parser;
use dsfb_database::adapters::postgres::load_pg_stat_statements;
use dsfb_database::grammar::{Episode, MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::non_claims;
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "tpc_c_generalization",
    about = "Phase-B2: replay a synthesised TPC-C pg_stat_statements snapshot through the unchanged Postgres adapter.",
    version
)]
struct Cli {
    /// RNG seed for the synthetic snapshot CSV.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Number of 60-second snapshots to synthesise.
    #[arg(long, default_value_t = 120)]
    n_snapshots: usize,
    /// Snapshot index at which the plan-regression onset is planted.
    #[arg(long, default_value_t = 60)]
    plan_onset_snap: usize,
    /// Magnitude multiplier applied to the hot-query's latency from
    /// `plan_onset_snap` onward.
    #[arg(long, default_value_t = 3.0)]
    plan_scale: f64,
    /// Snapshot index at which the workload-phase concentration is planted.
    #[arg(long, default_value_t = 80)]
    phase_onset_snap: usize,
    /// Output directory.
    #[arg(long, default_value = "out")]
    out: PathBuf,
}

/// TPC-C transaction mix, rounded from the TPC-C v5 spec table 4.1. We
/// split into 15 `query_id`s to reflect the fact that one transaction
/// issues several distinct SQL statements (NewOrder alone fires
/// ~15 statements; we compress to the busiest per type).
struct Qid {
    name: &'static str,
    /// Relative call share in the baseline window.
    share: f64,
    /// Baseline mean latency (ms / call) before any regression.
    base_ms: f64,
}

const TPCC_QIDS: &[Qid] = &[
    Qid {
        name: "neworder_select_stock",
        share: 0.22,
        base_ms: 0.18,
    },
    Qid {
        name: "neworder_update_stock",
        share: 0.18,
        base_ms: 0.22,
    },
    Qid {
        name: "neworder_insert_orderline",
        share: 0.14,
        base_ms: 0.15,
    },
    Qid {
        name: "payment_update_warehouse",
        share: 0.11,
        base_ms: 0.28,
    },
    Qid {
        name: "payment_update_customer",
        share: 0.10,
        base_ms: 0.31,
    },
    Qid {
        name: "payment_insert_history",
        share: 0.06,
        base_ms: 0.19,
    },
    Qid {
        name: "orderstatus_select_customer",
        share: 0.04,
        base_ms: 0.41,
    },
    Qid {
        name: "orderstatus_select_order",
        share: 0.03,
        base_ms: 0.37,
    },
    Qid {
        name: "delivery_select_neworder",
        share: 0.03,
        base_ms: 0.52,
    },
    Qid {
        name: "delivery_delete_neworder",
        share: 0.02,
        base_ms: 0.26,
    },
    Qid {
        name: "delivery_update_orderline",
        share: 0.02,
        base_ms: 0.44,
    },
    Qid {
        name: "stocklevel_select_stock",
        share: 0.02,
        base_ms: 0.88,
    },
    Qid {
        name: "stocklevel_select_orderline",
        share: 0.01,
        base_ms: 1.24,
    },
    Qid {
        name: "analytics_reporting",
        share: 0.01,
        base_ms: 3.10,
    },
    Qid {
        name: "sys_autovacuum",
        share: 0.01,
        base_ms: 0.05,
    },
];

/// Snapshot cadence in seconds. Mirrors the adapter's `BASELINE_WINDOW`
/// comment (3 intervals × 60 s = 180 s baseline).
const SNAPSHOT_DT_S: f64 = 60.0;

/// After `phase_onset_snap`, these three qids absorb almost all calls —
/// the rest of the mix drops to a residual trickle. The drop in entropy
/// is exactly the signal the workload-phase adapter translates into a
/// residual.
const PHASE_CONCENTRATORS: &[usize] = &[0, 3, 6];

/// Target total calls (across all qids) per snapshot in the baseline
/// window. Higher values make the share-distribution variance smaller
/// without affecting the motif signal.
const CALLS_PER_SNAPSHOT: u64 = 20_000;

/// RNG-jitter around each qid's mean-time-per-call, as a fraction of
/// the base. 8% keeps baseline-window noise well under the default
/// drift threshold.
const LATENCY_JITTER: f64 = 0.08;

struct SnapshotRow {
    snapshot_t: f64,
    query_id: String,
    calls: u64,
    total_exec_time_ms: f64,
}

fn synth_snapshots(cli: &Cli) -> Vec<SnapshotRow> {
    let mut rng = Pcg64::seed_from_u64(cli.seed);
    let mut cum_calls: Vec<u64> = vec![0; TPCC_QIDS.len()];
    let mut cum_exec_ms: Vec<f64> = vec![0.0; TPCC_QIDS.len()];
    let mut rows: Vec<SnapshotRow> = Vec::with_capacity(cli.n_snapshots * TPCC_QIDS.len());
    let t0: f64 = 1_700_000_000.0;
    for snap in 0..cli.n_snapshots {
        let snapshot_t = t0 + snap as f64 * SNAPSHOT_DT_S;
        let in_phase = snap >= cli.phase_onset_snap;
        let in_plan = snap >= cli.plan_onset_snap;
        let mut shares: Vec<f64> = TPCC_QIDS.iter().map(|q| q.share).collect();
        if in_phase {
            // Concentrate ~92% of calls onto PHASE_CONCENTRATORS.
            let concentrated_total: f64 = 0.92;
            let remainder: f64 = 1.0 - concentrated_total;
            let mut new_shares = vec![0.0; TPCC_QIDS.len()];
            let per_hot = concentrated_total / PHASE_CONCENTRATORS.len() as f64;
            for &i in PHASE_CONCENTRATORS {
                new_shares[i] = per_hot;
            }
            let cold_total: f64 = TPCC_QIDS
                .iter()
                .enumerate()
                .filter(|(i, _)| !PHASE_CONCENTRATORS.contains(i))
                .map(|(_, q)| q.share)
                .sum();
            if cold_total > 0.0 {
                for (i, q) in TPCC_QIDS.iter().enumerate() {
                    if !PHASE_CONCENTRATORS.contains(&i) {
                        new_shares[i] = remainder * q.share / cold_total;
                    }
                }
            }
            shares = new_shares;
        }
        for (i, q) in TPCC_QIDS.iter().enumerate() {
            let base_calls = (shares[i] * CALLS_PER_SNAPSHOT as f64).round() as u64;
            if base_calls == 0 {
                // Still emit a row so the adapter sees the qid; otherwise
                // the workload-phase entropy sum drops a degree-of-freedom
                // and the baseline / post-drift comparison becomes unfair.
                rows.push(SnapshotRow {
                    snapshot_t,
                    query_id: q.name.to_string(),
                    calls: cum_calls[i],
                    total_exec_time_ms: cum_exec_ms[i],
                });
                continue;
            }
            let jitter: f64 = rng.gen_range(-LATENCY_JITTER..LATENCY_JITTER);
            let mut per_call_ms = q.base_ms * (1.0 + jitter);
            if in_plan && i == 0 {
                per_call_ms *= cli.plan_scale;
            }
            let delta_calls = base_calls;
            let delta_exec_ms = per_call_ms * delta_calls as f64;
            cum_calls[i] += delta_calls;
            cum_exec_ms[i] += delta_exec_ms;
            rows.push(SnapshotRow {
                snapshot_t,
                query_id: q.name.to_string(),
                calls: cum_calls[i],
                total_exec_time_ms: cum_exec_ms[i],
            });
        }
    }
    rows
}

fn write_snapshot_csv(rows: &[SnapshotRow], path: &Path) -> Result<()> {
    let mut f = std::io::BufWriter::new(fs::File::create(path)?);
    writeln!(f, "snapshot_t,query_id,calls,total_exec_time_ms")?;
    for r in rows {
        writeln!(
            f,
            "{:.6},{},{},{:.6}",
            r.snapshot_t, r.query_id, r.calls, r.total_exec_time_ms
        )?;
    }
    f.flush()?;
    Ok(())
}

/// Windows of planted ground truth, in absolute `snapshot_t − t0`
/// (relative-seconds), because the adapter rebases to `t0 = min snapshot_t`.
fn ground_truth_windows(cli: &Cli) -> Vec<(MotifClass, f64, f64)> {
    let end_t = (cli.n_snapshots as f64 - 1.0) * SNAPSHOT_DT_S;
    vec![
        (
            MotifClass::PlanRegressionOnset,
            cli.plan_onset_snap as f64 * SNAPSHOT_DT_S,
            end_t,
        ),
        (
            MotifClass::WorkloadPhaseTransition,
            cli.phase_onset_snap as f64 * SNAPSHOT_DT_S,
            end_t,
        ),
    ]
}

fn first_overlapping(
    episodes: &[Episode],
    motif: MotifClass,
    window: (f64, f64),
) -> Option<&Episode> {
    let (on, off) = window;
    episodes
        .iter()
        .filter(|e| e.motif == motif && e.t_end >= on && e.t_start <= off)
        .min_by(|a, b| {
            a.t_start
                .partial_cmp(&b.t_start)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn count_by_motif(episodes: &[Episode], motif: MotifClass) -> usize {
    episodes.iter().filter(|e| e.motif == motif).count()
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    non_claims::print();

    fs::create_dir_all(&cli.out)?;
    let rows = synth_snapshots(&cli);
    let input_path = cli.out.join("tpc_c_generalization_input.csv");
    write_snapshot_csv(&rows, &input_path)?;

    let stream = load_pg_stat_statements(&input_path)?;
    let episodes = MotifEngine::new(MotifGrammar::default()).run(&stream);

    let gt = ground_truth_windows(&cli);
    let csv_path = cli.out.join("tpc_c_generalization.csv");
    let mut wtr = csv::Writer::from_path(&csv_path)?;
    wtr.write_record([
        "motif",
        "seed",
        "n_snapshots",
        "plan_onset_snap",
        "plan_scale",
        "phase_onset_snap",
        "gt_onset_s",
        "gt_end_s",
        "episodes_this_motif",
        "detected",
        "ttd_s",
        "episode_peak",
        "episode_ema_at_boundary",
    ])?;
    for motif in MotifClass::ALL {
        let this_gt = gt.iter().find(|(m, _, _)| *m == motif).copied();
        let count = count_by_motif(&episodes, motif);
        let (gt_on_s, gt_end_s, detected, ttd_s, peak, ema) = match this_gt {
            Some((_, on, off)) => {
                let ep = first_overlapping(&episodes, motif, (on, off));
                match ep {
                    Some(e) => (
                        on,
                        off,
                        true,
                        (e.t_start - on).max(0.0),
                        e.peak,
                        e.ema_at_boundary,
                    ),
                    None => (on, off, false, f64::NAN, f64::NAN, f64::NAN),
                }
            }
            None => (f64::NAN, f64::NAN, false, f64::NAN, f64::NAN, f64::NAN),
        };
        let fmt = |x: f64| {
            if x.is_nan() {
                "nan".to_string()
            } else {
                format!("{:.6}", x)
            }
        };
        wtr.write_record([
            motif.name(),
            &cli.seed.to_string(),
            &cli.n_snapshots.to_string(),
            &cli.plan_onset_snap.to_string(),
            &format!("{:.3}", cli.plan_scale),
            &cli.phase_onset_snap.to_string(),
            &fmt(gt_on_s),
            &fmt(gt_end_s),
            &count.to_string(),
            &detected.to_string(),
            &fmt(ttd_s),
            &fmt(peak),
            &fmt(ema),
        ])?;
    }
    wtr.flush()?;

    eprintln!(
        "tpc_c_generalization: {} residuals, {} episodes total, wrote {}",
        stream.samples.len(),
        episodes.len(),
        csv_path.display()
    );
    for motif in MotifClass::ALL {
        let count = count_by_motif(&episodes, motif);
        eprintln!("  {}: {} episode(s)", motif.name(), count);
    }
    Ok(())
}
