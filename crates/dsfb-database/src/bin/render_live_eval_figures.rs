#![forbid(unsafe_code)]

//! Render the paper's §Live Evaluation figures from the pinned tape
//! fixtures at `paper/fixtures/live_pg_real/`.
//!
//! Produces two PNGs:
//!   * `paper/figs/live_real_pg_trajectory.png` — three-panel
//!     trajectory from `replication_01.tape.jsonl` + `ground_truth.json`
//!     + DSFB/ADWIN/BOCPD/PELT detections on the primary fault qid.
//!   * `paper/figs/live_determinism_overlay.png` — two-panel overlay
//!     of `replication_01` and `replication_02` showing engine→tape
//!     divergence (Panel A) and tape→episodes byte-stability (Panel B,
//!     via per-tape episode fingerprints).
//!
//! Both figures are byte-deterministic functions of the pinned
//! fixtures. Re-running this binary against the same fixtures
//! produces byte-identical PNGs.

use anyhow::{Context, Result};
use clap::Parser;
use dsfb_database::baselines::{
    adwin::Adwin, bocpd::Bocpd, pelt::Pelt, run_detector, ChangePointDetector,
};
use dsfb_database::grammar::{Episode, MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::live::tape::load_and_verify;
use dsfb_database::report::plots_live::{
    plot_live_determinism_overlay, plot_live_real_pg, CacheBucketTrace, DetectorMark, EpisodeRect,
    QidTrace,
};
use dsfb_database::residual::{ResidualClass, ResidualStream};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "render_live_eval_figures", version)]
struct Cli {
    /// Directory with replication_01/02 tape + hash + poll_log + ground_truth.
    #[arg(long)]
    fixtures_dir: PathBuf,
    /// Output directory for rendered PNGs.
    #[arg(long)]
    figs_dir: PathBuf,
}

#[derive(Deserialize)]
struct GroundTruthWindow {
    motif: String,
    channel: String,
    t_start: f64,
    t_end: f64,
}

#[derive(Deserialize)]
struct GroundTruth {
    #[allow(dead_code)]
    tape_sha256: String,
    #[allow(dead_code)]
    fault_description: String,
    windows: Vec<GroundTruthWindow>,
}

fn load_poll_log(path: &Path) -> Result<(Vec<f64>, Vec<f64>)> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening {}", path.display()))?;
    let mut t = Vec::new();
    let mut throttle = Vec::new();
    let mut t0 = None;
    for rec in rdr.records() {
        let rec = rec?;
        let t_wall: f64 = rec.get(0).unwrap_or("0").parse().unwrap_or(0.0);
        let tf: f64 = rec.get(3).unwrap_or("1").parse().unwrap_or(1.0);
        if t0.is_none() {
            t0 = Some(t_wall);
        }
        t.push(t_wall - t0.unwrap_or(t_wall));
        throttle.push(tf);
    }
    Ok((t, throttle))
}

fn group_by_channel(
    stream: &ResidualStream,
    class: ResidualClass,
) -> BTreeMap<String, (Vec<f64>, Vec<f64>)> {
    let mut by_ch: BTreeMap<String, (Vec<f64>, Vec<f64>)> = BTreeMap::new();
    for s in stream.iter_class(class) {
        let ch = s.channel.clone().unwrap_or_else(|| "<none>".to_string());
        let e = by_ch.entry(ch).or_default();
        e.0.push(s.t);
        e.1.push(s.value);
    }
    by_ch
}

fn top_k_by_peak(
    mut channels: Vec<(String, (Vec<f64>, Vec<f64>))>,
    k: usize,
    required: &[&str],
) -> Vec<(String, (Vec<f64>, Vec<f64>))> {
    channels.sort_by(|a, b| {
        let pa = a.1 .1.iter().cloned().fold(0.0_f64, f64::max);
        let pb = b.1 .1.iter().cloned().fold(0.0_f64, f64::max);
        pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
    });
    // guarantee the required fault channels are included, then fill with top-k
    let mut out: Vec<(String, (Vec<f64>, Vec<f64>))> = Vec::new();
    for r in required {
        if let Some(pos) = channels.iter().position(|(ch, _)| ch == r) {
            out.push(channels.remove(pos));
        }
    }
    for c in channels.into_iter().take(k.saturating_sub(out.len())) {
        out.push(c);
    }
    out
}

fn short_channel(ch: &str) -> String {
    if ch.len() > 10 {
        format!("{}…", &ch[..8])
    } else {
        ch.to_string()
    }
}

fn detect_earliest_on_channel(
    detector: &dyn ChangePointDetector,
    stream: &ResidualStream,
    channel: &str,
) -> Option<f64> {
    let mut best = None;
    for m in MotifClass::ALL {
        for ep in run_detector(detector, m, stream) {
            if ep.channel.as_deref() == Some(channel) {
                let t = ep.t_start;
                best = Some(best.map_or(t, |b: f64| b.min(t)));
            }
        }
    }
    best
}

fn dsfb_episodes(stream: &ResidualStream) -> Vec<Episode> {
    let engine = MotifEngine::new(MotifGrammar::default());
    engine.run(stream)
}

fn episode_fingerprint(eps: &[Episode]) -> String {
    // Byte-stable episode-set hash: same tape → same episodes →
    // same fingerprint. Used on the determinism-overlay figure to
    // annotate that each tape's replay is byte-stable.
    let mut h = Sha256::new();
    // Sort by (t_start, motif name, channel) for order-independence.
    let mut sorted: Vec<&Episode> = eps.iter().collect();
    sorted.sort_by(|a, b| {
        a.t_start
            .partial_cmp(&b.t_start)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.motif.name().cmp(b.motif.name()))
            .then_with(|| a.channel.cmp(&b.channel))
    });
    for e in sorted {
        h.update(e.motif.name().as_bytes());
        h.update(b"|");
        h.update(e.channel.as_deref().unwrap_or("").as_bytes());
        h.update(b"|");
        h.update(e.t_start.to_le_bytes());
        h.update(e.t_end.to_le_bytes());
        h.update(e.peak.to_le_bytes());
    }
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn render_real_pg(
    fixtures_dir: &Path,
    figs_dir: &Path,
) -> Result<()> {
    let tape_path = fixtures_dir.join("replication_01.tape.jsonl");
    let gt_path = fixtures_dir.join("ground_truth.json");
    let poll_path = fixtures_dir.join("replication_01.poll_log.csv");

    let (stream, manifest) = load_and_verify(&tape_path)?;
    let gt: GroundTruth = serde_json::from_slice(&fs::read(&gt_path)?)?;
    let gt_window = gt
        .windows
        .iter()
        .find(|w| w.motif == "plan_regression_onset")
        .map(|w| (w.t_start, w.t_end));
    let primary_channel = gt
        .windows
        .iter()
        .find(|w| w.motif == "plan_regression_onset")
        .map(|w| w.channel.clone())
        .unwrap_or_default();

    // --- plan_regression traces: primary fault qid + top-k by peak ---
    let plan_groups = group_by_channel(&stream, ResidualClass::PlanRegression);
    let plan_vec: Vec<(String, (Vec<f64>, Vec<f64>))> = plan_groups.into_iter().collect();
    let required = [primary_channel.as_str()];
    let chosen = top_k_by_peak(plan_vec, 5, &required);
    let labels: Vec<String> = chosen
        .iter()
        .map(|(ch, _)| {
            if ch == &primary_channel {
                format!("fault qid {} (SELECT abalance)", short_channel(ch))
            } else if ch == "793978ca6e24c91f0a87f8cc8020d232" {
                format!("pgbench UPDATE {}", short_channel(ch))
            } else {
                format!("qid {}", short_channel(ch))
            }
        })
        .collect();
    let plan_traces: Vec<QidTrace<'_>> = chosen
        .iter()
        .zip(&labels)
        .map(|((_, (t, v)), lab)| QidTrace {
            label: lab.as_str(),
            t: t.as_slice(),
            v: v.as_slice(),
        })
        .collect();

    // --- cache_io traces: up to 4 buckets with largest mean value ---
    let mut cache_groups: Vec<(String, (Vec<f64>, Vec<f64>))> =
        group_by_channel(&stream, ResidualClass::CacheIo).into_iter().collect();
    cache_groups.sort_by(|a, b| {
        let ma = a.1 .1.iter().cloned().sum::<f64>() / a.1 .1.len().max(1) as f64;
        let mb = b.1 .1.iter().cloned().sum::<f64>() / b.1 .1.len().max(1) as f64;
        mb.partial_cmp(&ma).unwrap_or(std::cmp::Ordering::Equal)
    });
    let cache_chosen: Vec<(String, (Vec<f64>, Vec<f64>))> =
        cache_groups.into_iter().take(4).collect();
    let cache_labels: Vec<String> = cache_chosen.iter().map(|(ch, _)| ch.clone()).collect();
    let cache_traces: Vec<CacheBucketTrace<'_>> = cache_chosen
        .iter()
        .zip(&cache_labels)
        .map(|((_, (t, v)), lab)| CacheBucketTrace {
            label: lab.as_str(),
            t: t.as_slice(),
            v: v.as_slice(),
        })
        .collect();

    // --- poll log (throttle factor) ---
    let (poll_t, throttle) = if poll_path.exists() {
        load_poll_log(&poll_path)?
    } else {
        (Vec::new(), Vec::new())
    };

    // --- per-detector detection timestamps on primary fault qid ---
    let adwin_mark = detect_earliest_on_channel(&Adwin::default(), &stream, &primary_channel);
    let bocpd_mark = detect_earliest_on_channel(&Bocpd::default(), &stream, &primary_channel);
    let pelt_mark = detect_earliest_on_channel(&Pelt::default(), &stream, &primary_channel);
    let dsfb_eps = dsfb_episodes(&stream);
    let dsfb_mark = dsfb_eps
        .iter()
        .filter(|e| e.channel.as_deref() == Some(primary_channel.as_str()))
        .map(|e| e.t_start)
        .fold(None, |b: Option<f64>, t| {
            Some(b.map_or(t, |x| x.min(t)))
        });

    let marks: Vec<DetectorMark<'_>> = vec![
        DetectorMark { label: "dsfb-database", t: dsfb_mark },
        DetectorMark { label: "adwin", t: adwin_mark },
        DetectorMark { label: "bocpd", t: bocpd_mark },
        DetectorMark { label: "pelt", t: pelt_mark },
    ];

    // --- DSFB episodes as rectangles ---
    let motif_names: Vec<String> =
        dsfb_eps.iter().map(|e| e.motif.name().to_string()).collect();
    let episodes: Vec<EpisodeRect<'_>> = dsfb_eps
        .iter()
        .zip(&motif_names)
        .map(|(e, n)| EpisodeRect {
            motif: n.as_str(),
            t_start: e.t_start,
            t_end: e.t_end,
        })
        .collect();

    let caption = format!(
        "§Live Evaluation — real engine PG 17, pgbench scale-10, c=16 j=4, 70s. \
         Fault: ALTER TABLE pgbench_accounts DROP CONSTRAINT pgbench_accounts_pkey at t=30s. \
         Replication 1/10 (tape SHA {}…). Dashed reference lines: motif thresholds. \
         Grey band: ground-truth fault window.",
        &manifest.sha256[..10]
    );

    fs::create_dir_all(figs_dir)?;
    let out = figs_dir.join("live_real_pg_trajectory.png");
    plot_live_real_pg(
        &out,
        &plan_traces,
        &cache_traces,
        &poll_t,
        &throttle,
        gt_window,
        &marks,
        &episodes,
        &caption,
    )?;
    eprintln!("wrote {}", out.display());
    Ok(())
}

fn render_overlay(fixtures_dir: &Path, figs_dir: &Path) -> Result<()> {
    let tape_a = fixtures_dir.join("replication_01.tape.jsonl");
    let tape_b = fixtures_dir.join("replication_02.tape.jsonl");
    let gt_path = fixtures_dir.join("ground_truth.json");

    let (stream_a, manifest_a) = load_and_verify(&tape_a)?;
    let (stream_b, manifest_b) = load_and_verify(&tape_b)?;
    let gt: GroundTruth = serde_json::from_slice(&fs::read(&gt_path)?)?;
    let primary_channel = gt
        .windows
        .iter()
        .find(|w| w.motif == "plan_regression_onset")
        .map(|w| w.channel.clone())
        .unwrap_or_default();

    let (ta_t, ta_v): (Vec<f64>, Vec<f64>) = stream_a
        .iter_class(ResidualClass::PlanRegression)
        .filter(|s| s.channel.as_deref() == Some(primary_channel.as_str()))
        .map(|s| (s.t, s.value))
        .unzip();
    let (tb_t, tb_v): (Vec<f64>, Vec<f64>) = stream_b
        .iter_class(ResidualClass::PlanRegression)
        .filter(|s| s.channel.as_deref() == Some(primary_channel.as_str()))
        .map(|s| (s.t, s.value))
        .unzip();
    let label_a = format!("replication_01 (qid {})", short_channel(&primary_channel));
    let label_b = format!("replication_02 (qid {})", short_channel(&primary_channel));
    let trace_a = QidTrace {
        label: label_a.as_str(),
        t: ta_t.as_slice(),
        v: ta_v.as_slice(),
    };
    let trace_b = QidTrace {
        label: label_b.as_str(),
        t: tb_t.as_slice(),
        v: tb_v.as_slice(),
    };

    let eps_a = dsfb_episodes(&stream_a);
    let eps_b = dsfb_episodes(&stream_b);
    let fp_a = episode_fingerprint(&eps_a);
    let fp_b = episode_fingerprint(&eps_b);

    let names_a: Vec<String> = eps_a.iter().map(|e| e.motif.name().to_string()).collect();
    let names_b: Vec<String> = eps_b.iter().map(|e| e.motif.name().to_string()).collect();
    let rects_a: Vec<EpisodeRect<'_>> = eps_a
        .iter()
        .zip(&names_a)
        .map(|(e, n)| EpisodeRect {
            motif: n.as_str(),
            t_start: e.t_start,
            t_end: e.t_end,
        })
        .collect();
    let rects_b: Vec<EpisodeRect<'_>> = eps_b
        .iter()
        .zip(&names_b)
        .map(|(e, n)| EpisodeRect {
            motif: n.as_str(),
            t_start: e.t_start,
            t_end: e.t_end,
        })
        .collect();

    let caption = format!(
        "§Live Evaluation, determinism asymmetry. \
         Both tapes captured under pgbench scale-10 + identical planted \
         DROP CONSTRAINT fault at t=30s; they diverge (Panel A) because the \
         engine→tape path depends on jitter-level timing. Each tape's \
         replay emits a byte-stable episode stream (Panel B) — this is \
         the 7th non-claim's determinism contract."
    );

    fs::create_dir_all(figs_dir)?;
    let out = figs_dir.join("live_determinism_overlay.png");
    plot_live_determinism_overlay(
        &out,
        "tape A",
        &trace_a,
        &rects_a,
        &manifest_a.sha256[..10],
        &fp_a[..10],
        "tape B",
        &trace_b,
        &rects_b,
        &manifest_b.sha256[..10],
        &fp_b[..10],
        &caption,
    )?;
    eprintln!("wrote {}", out.display());
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    render_real_pg(&cli.fixtures_dir, &cli.figs_dir)?;
    render_overlay(&cli.fixtures_dir, &cli.figs_dir)?;
    Ok(())
}
