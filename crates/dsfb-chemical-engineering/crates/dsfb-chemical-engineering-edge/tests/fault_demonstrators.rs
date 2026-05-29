//! Reduction-to-practice gate for the F1 (valve stiction) and F9 (valve hunting) fault signatures.
//!
//! The atlas records F1 and F9 as `Executed` — meaning the project actually runs a demonstrator that
//! exhibits the fault's residual motif through the detector bank, not merely catalogues it. This test
//! IS that execution: it loads the committed synthetic demonstrators
//! (`data/instrumented/valve_{stiction,hunting}_instrumented.csv`, produced by
//! `scripts/gen_instrumented.py`), runs the full pipeline, and asserts that (i) a single fused
//! structural episode forms at the labelled fault onset, (ii) the baseline stays quiet, and (iii) the
//! signature's named detector inputs fire densely in the fault region. If this regresses, the atlas
//! `Executed` claim for F1/F9 is no longer true and the gate fails.

use dsfb_chemical_engineering_edge as edge;
use edge::datasets;
use edge::pipeline::{self, PipelineConfig};
use std::path::PathBuf;

const ONSET: usize = 360; // must match scripts/gen_instrumented.py (onset=360, n=720)
const N: usize = 720;

fn slice_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("instrumented")
        .join(format!("{name}.csv"))
}

fn run(
    name: &str,
) -> (
    pipeline::AnalysisResult,
    Vec<edge::fusion::DetectorTimeline>,
) {
    let (m, n_base) = datasets::load_csv_slice(&slice_path(name), 0.4).expect("load demonstrator");
    assert_eq!(m.n_samples, N, "{name} should have {N} samples");
    assert_eq!(m.fault_onset, Some(ONSET), "{name} fault onset");
    let cfg = PipelineConfig::default();
    let res = pipeline::analyze(name, "synthetic", &m, n_base, cfg);
    let timelines = pipeline::timelines_for(name, &m, n_base, cfg);
    (res, timelines)
}

/// Count breaches of detector `id` at or after `from`.
fn breaches_after(timelines: &[edge::fusion::DetectorTimeline], id: &str, from: usize) -> usize {
    timelines
        .iter()
        .find(|t| t.detector_id == id)
        .map(|t| {
            t.outputs
                .iter()
                .filter(|o| o.breach && o.time_index >= from)
                .count()
        })
        .unwrap_or(0)
}

/// Shared assertions: exactly one fused episode, at the onset, covering the fault region; and each
/// named detector fires densely (>= 100 of the ~360 fault samples) within the fault region.
fn assert_executed(name: &str, detectors: &[&str]) {
    let (res, timelines) = run(name);

    // A single fused episode: the baseline is quiet (no spurious pre-onset episodes) and the fault
    // forms one coherent structural interval — the cleanest possible reduction-to-practice.
    assert_eq!(
        res.fused_episodes.len(),
        1,
        "{name}: expected exactly one fused episode"
    );
    let ep = &res.fused_episodes[0];
    assert!(
        ep.start_index >= ONSET.saturating_sub(5) && ep.start_index <= ONSET + 5,
        "{name}: episode should start at the fault onset (got {})",
        ep.start_index
    );
    assert!(
        ep.end_index >= N - 10,
        "{name}: episode should cover the fault region to the end (got {})",
        ep.end_index
    );

    // Detection delay is small in magnitude (detected within a few samples of the labelled onset).
    let delay = res.metrics.detection_delay.expect("labelled onset present");
    assert!(
        delay.abs() <= 5,
        "{name}: detection delay should be near zero (got {delay})"
    );

    // The signature's named detector inputs each fire densely in the fault region.
    for id in detectors {
        let n = breaches_after(&timelines, id, ONSET);
        assert!(
            n >= 100,
            "{name}: detector {id} should fire densely in the fault region (got {n})"
        );
    }
}

#[test]
fn f1_valve_stiction_is_executed() {
    // F1 detector_inputs in the atlas: page_hinkley_spe, cusum_spe, spectral_entropy_spe.
    assert_executed(
        "valve_stiction_instrumented",
        &["page_hinkley_spe", "cusum_spe", "spectral_entropy_spe"],
    );
}

#[test]
fn f9_valve_hunting_is_executed() {
    // F9 detector_inputs in the atlas: spectral_entropy_spe, page_hinkley_spe.
    assert_executed(
        "valve_hunting_instrumented",
        &["spectral_entropy_spe", "page_hinkley_spe"],
    );
}

#[test]
fn f2_cavitation_is_executed() {
    // F2 detector_inputs in the atlas: spectral_entropy_spe, knn_spe, ewma_spe. Cavitation onset is
    // ABRUPT (suction conditions cross into vapour formation), so this uses the strict gate (one
    // onset episode, delay ≈ 0) like F1/F9. The flow dither dominates PCA (k=1), leaving vibration as
    // the SPE residual: a tonal hum in the baseline, broadband white energy after onset. That drives
    // ewma_spe (sustained magnitude), knn_spe (far from the baseline SPE cloud) and spectral_entropy_spe
    // (tonal→broadband shape change) densely through the fault region.
    assert_executed(
        "cavitation_instrumented",
        &["spectral_entropy_spe", "knn_spe", "ewma_spe"],
    );
}

/// Relaxed gate for GRADUAL faults (F3 fouling drift, F8 controller masking). Unlike the abrupt F1/F9
/// stick-slip/limit-cycle (onset episode, delay ≈ 0), a slow monotone drift or a latent masked change is
/// legitimately detected with a *bounded delay* and may leave a small baseline blip from the conditioning
/// dither — asserting delay ≈ 0 / exactly-one-episode would be physically wrong here. So we assert: a
/// fused episode covers the fault region to the end; detection delay is within `max_delay` (and not
/// early); the baseline stays mostly quiet (no large pre-onset episode); and the named detectors fire
/// densely after onset.
fn assert_fault_detected(name: &str, detectors: &[&str], max_delay: i64) {
    let (res, timelines) = run(name);
    // A fused episode covers the fault region through to the end.
    let cover = res.fused_episodes.iter().find(|e| e.end_index >= N - 10);
    let ep = cover.unwrap_or_else(|| panic!("{name}: no fused episode covering the fault region"));
    assert!(
        ep.start_index >= ONSET,
        "{name}: fault episode should start at/after onset (got {})",
        ep.start_index
    );
    // Detection delay is bounded and not early (a gradual fault is detected once it clears the band).
    let delay = res.metrics.detection_delay.expect("labelled onset present");
    assert!(
        (0..=max_delay).contains(&delay),
        "{name}: detection delay {delay} not in 0..={max_delay}"
    );
    // Baseline mostly quiet: no pre-onset episode of more than a few steps (small dither blips allowed).
    let big_baseline = res
        .fused_episodes
        .iter()
        .any(|e| e.end_index < ONSET && e.step_count > 15);
    assert!(
        !big_baseline,
        "{name}: a large pre-onset episode formed (baseline not quiet)"
    );
    // The signature's named detector inputs fire densely in the fault region.
    for id in detectors {
        let n = breaches_after(&timelines, id, ONSET);
        assert!(
            n >= 100,
            "{name}: detector {id} should fire densely in the fault region (got {n})"
        );
    }
}

#[test]
fn f3_heat_fouling_is_executed() {
    // F3 detector_inputs in the atlas: ewma_spe, mann_kendall_spe, co_drift. The slow monotone fouling
    // drift lands as a sustained SPE shift that ewma_spe tracks densely (mann_kendall fires at the drift
    // onset; co_drift needs ≥3 co-drifting variables, which this 2-channel demonstrator does not provide).
    assert_fault_detected("heat_fouling_instrumented", &["ewma_spe"], 30);
}

#[test]
fn f8_controller_masking_is_executed() {
    // F8 detector_inputs in the atlas: pca_t2, ewma_spe, co_drift. The controller holds CV in-band while
    // MV ramps to reject the growing load — a latent T²/SPE structural change that pca_t2 and ewma_spe
    // both catch densely before any raw CV-limit breach (co_drift again needs ≥3 variables).
    assert_fault_detected(
        "controller_masking_instrumented",
        &["pca_t2", "ewma_spe"],
        55,
    );
}
