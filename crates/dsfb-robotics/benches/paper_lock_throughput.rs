//! Streaming-throughput benchmark for the DSFB engine.
//!
//! Measures the per-sample cost of `DsfbRoboticsEngine::observe` on
//! every dataset in the slate so a downstream practitioner can decide
//! whether DSFB is feasible to run inline at the residual production
//! rate of their incumbent observer, on every published row.
//!
//! Run with:
//!     cargo bench --bench paper_lock_throughput --features std,paper_lock
//!
//! Criterion emits per-dataset timings to stdout and to
//! `target/criterion/paper_lock_<slug>/`; downstream scripts can scrape
//! the JSON outputs for paper-table generation.
#![cfg(all(feature = "std", feature = "paper_lock"))]

use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use dsfb_robotics::datasets::DatasetId;
use dsfb_robotics::paper_lock::run_real_data_with_csv_path;

fn locate_csv(slug: &str) -> Option<PathBuf> {
    let p = PathBuf::from(format!("data/processed/{slug}_published.csv"));
    if p.is_file() {
        return Some(p);
    }
    let p2 = PathBuf::from(format!("data/processed/{slug}.csv"));
    if p2.is_file() {
        return Some(p2);
    }
    None
}

fn count_samples(csv_path: &PathBuf) -> usize {
    let raw = std::fs::read_to_string(csv_path).expect("read CSV");
    raw.lines().filter(|l| !l.is_empty()).count().saturating_sub(1)
}

const SLUGS_AND_IDS: &[(&str, DatasetId)] = &[
    ("cwru", DatasetId::Cwru),
    ("ims", DatasetId::Ims),
    ("kuka_lwr", DatasetId::KukaLwr),
    ("femto_st", DatasetId::FemtoSt),
    ("panda_gaz", DatasetId::PandaGaz),
    ("dlr_justin", DatasetId::DlrJustin),
    ("ur10_kufieta", DatasetId::Ur10Kufieta),
    ("cheetah3", DatasetId::Cheetah3),
    ("icub_pushrecovery", DatasetId::IcubPushRecovery),
    ("droid", DatasetId::Droid),
    ("openx", DatasetId::Openx),
    ("anymal_parkour", DatasetId::AnymalParkour),
    ("unitree_g1", DatasetId::UnitreeG1),
    ("aloha_static", DatasetId::AlohaStatic),
    ("icub3_sorrentino", DatasetId::Icub3Sorrentino),
    ("mobile_aloha", DatasetId::MobileAloha),
    ("so100", DatasetId::So100),
    ("aloha_static_tape", DatasetId::AlohaStaticTape),
    ("aloha_static_screw_driver", DatasetId::AlohaStaticScrewDriver),
    ("aloha_static_pingpong_test", DatasetId::AlohaStaticPingpongTest),
];

fn bench_all_datasets(c: &mut Criterion) {
    for (slug, id) in SLUGS_AND_IDS {
        let Some(csv_path) = locate_csv(slug) else {
            eprintln!("bench: skipping {slug} (no CSV)");
            continue;
        };
        let n_samples = count_samples(&csv_path);
        let mut group = c.benchmark_group(format!("paper_lock_{slug}"));
        // Smaller datasets: Criterion default sample size; larger: 20.
        if n_samples > 50_000 {
            group.sample_size(15);
        } else {
            group.sample_size(20);
        }
        group.throughput(Throughput::Elements(n_samples as u64));
        let id = *id;
        group.bench_function("end_to_end", |b| {
            b.iter(|| {
                let report = run_real_data_with_csv_path(id, false, &csv_path).unwrap();
                criterion::black_box(report.aggregate.total_samples)
            });
        });
        group.finish();
    }
}

criterion_group!(benches, bench_all_datasets);
criterion_main!(benches);
