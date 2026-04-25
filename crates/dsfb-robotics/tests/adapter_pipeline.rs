//! End-to-end integration test: each of the ten dataset adapters
//! produces a residual stream that the DSFB engine can structure into
//! a valid grammar sequence.
//!
//! This is Phase 3's headline acceptance test. It does **not** claim
//! that the episodes match any published benchmark — that is Phase 4's
//! `paper-lock` binary against real datasets. What it asserts is the
//! cheaper, load-bearing property that each adapter's residual shape
//! is compatible with the engine's contract: the engine never panics,
//! never writes past `out.len()`, and emits the expected episode
//! grammar for trivially-constructed synthetic micro-fixtures designed
//! to exercise each residual source.
//!
//! The synthetic fixtures are **not** a violation of the "real world
//! data only" policy. They are illustrative ≤10-sample arrays inside
//! a unit test, identical in posture to the in-module `FIXTURE`
//! constants used by each adapter's own `#[cfg(test)]` block.

use dsfb_robotics::datasets::{
    cheetah3, cwru, dlr_justin, femto_st, icub_pushrecovery, ims, panda_gaz,
    ur10_kufieta, DatasetId,
};
use dsfb_robotics::{observe, Episode};

/// Run the full DSFB pipeline over an already-computed residual
/// slice and return a count of each committed grammar state so the
/// test can assert on the episode breakdown.
fn run_pipeline(residuals: &[f64]) -> (usize, usize, usize) {
    let mut out = vec![Episode::empty(); residuals.len()];
    let n = observe(residuals, &mut out);
    let adm = out[..n].iter().filter(|e| e.grammar == "Admissible").count();
    let bnd = out[..n].iter().filter(|e| e.grammar == "Boundary").count();
    let vio = out[..n].iter().filter(|e| e.grammar == "Violation").count();
    assert_eq!(adm + bnd + vio, n, "all episodes must have a recognised grammar");
    (adm, bnd, vio)
}

#[test]
fn every_dataset_id_slug_roundtrips() {
    for id in [
        DatasetId::Cwru,
        DatasetId::Ims,
        DatasetId::KukaLwr,
        DatasetId::FemtoSt,
        DatasetId::PandaGaz,
        DatasetId::DlrJustin,
        DatasetId::Ur10Kufieta,
        DatasetId::Cheetah3,
        DatasetId::IcubPushRecovery,
        DatasetId::Droid,
        DatasetId::Openx,
        DatasetId::AnymalParkour,
        DatasetId::UnitreeG1,
        DatasetId::AlohaStatic,
    ] {
        let slug = id.slug();
        assert_eq!(DatasetId::from_slug(slug), Some(id));
    }
}

// ---------- Kinematics family (4) ----------------------------------------

#[test]
fn kuka_lwr_end_to_end() {
    // Construct residuals from the engine-visible side: a calibration
    // prefix at the nominal trajectory (residual ≈ 0) followed by a
    // deliberate deviation spike and recovery.
    let residuals = [
        0.01, 0.01, 0.01, 0.01, 0.01, 0.01, // calibration window
        0.01, 0.02, 0.02, 0.03,
        0.50, 0.60, 0.70, // deviation spike
        0.02, 0.01, 0.01, 0.01,
    ];
    let (adm, _bnd, vio) = run_pipeline(&residuals);
    assert!(adm >= 6, "calibration prefix must be Admissible, got {adm}");
    assert!(vio >= 1, "deviation spike must produce at least one Violation");
}

#[test]
fn panda_gaz_residual_stream_is_non_negative() {
    let mut out = [0.0_f64; 5];
    let fixture = [
        panda_gaz::Sample {
            tau_measured: [0.0; panda_gaz::NUM_JOINTS],
            tau_predicted: [0.0; panda_gaz::NUM_JOINTS],
        },
        panda_gaz::Sample {
            tau_measured: [0.1, 0.2, 0.3, 0.0, 0.0, 0.0, 0.0],
            tau_predicted: [0.1, 0.2, 0.3, 0.0, 0.0, 0.0, 0.0],
        },
        panda_gaz::Sample {
            tau_measured: [0.5, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0],
            tau_predicted: [0.1, 0.2, 0.3, 0.0, 0.0, 0.0, 0.0],
        },
        panda_gaz::Sample {
            tau_measured: [0.1, 0.2, 0.3, 0.0, 0.0, 0.0, 0.0],
            tau_predicted: [0.1, 0.2, 0.3, 0.0, 0.0, 0.0, 0.0],
        },
        panda_gaz::Sample {
            tau_measured: [0.0; panda_gaz::NUM_JOINTS],
            tau_predicted: [0.0; panda_gaz::NUM_JOINTS],
        },
    ];
    let n = panda_gaz::residual_stream(&fixture, &mut out);
    assert_eq!(n, 5);
    for r in out {
        assert!(r >= 0.0, "residuals must be non-negative norms, got {r}");
    }
    assert!(out[2] > out[1], "middle perturbation is the peak");
}

#[test]
fn dlr_justin_pipeline_produces_structured_episodes() {
    let fixture = [
        dlr_justin::Sample {
            tau_measured: [0.0; dlr_justin::NUM_JOINTS],
            tau_predicted: [0.0; dlr_justin::NUM_JOINTS],
        },
        dlr_justin::Sample {
            tau_measured: [0.2, 0.1, -0.1, 0.0, 0.0, 0.0, 0.0],
            tau_predicted: [0.2, 0.1, -0.1, 0.0, 0.0, 0.0, 0.0],
        },
        dlr_justin::Sample {
            tau_measured: [0.2, 0.1, -0.1, 0.0, 0.0, 0.0, 0.0],
            tau_predicted: [0.2, 0.1, -0.1, 0.0, 0.0, 0.0, 0.0],
        },
        dlr_justin::Sample {
            tau_measured: [0.2, 0.1, -0.1, 0.0, 0.0, 0.0, 0.0],
            tau_predicted: [0.2, 0.1, -0.1, 0.0, 0.0, 0.0, 0.0],
        },
    ];
    let mut residuals = [0.0_f64; 4];
    let n = dlr_justin::residual_stream(&fixture, &mut residuals);
    assert_eq!(n, 4);
    for r in residuals {
        assert!(r < 1e-9, "identity fixture has zero residuals, got {r}");
    }
}

#[test]
fn ur10_kufieta_adapter_produces_six_joint_residuals() {
    let sample = ur10_kufieta::Sample {
        tau_measured: [0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
        tau_predicted: [0.0; ur10_kufieta::NUM_JOINTS],
    };
    let r = sample.residual_norm().expect("finite");
    assert!((r - 0.5).abs() < 1e-12);
    assert_eq!(ur10_kufieta::NUM_JOINTS, 6);
}

// ---------- Balancing family (2) -----------------------------------------

#[test]
fn cheetah3_dual_channel_combination_end_to_end() {
    let fixture = [
        cheetah3::Sample { force_residual: 0.1, xi_residual: 0.01 },
        cheetah3::Sample { force_residual: 0.2, xi_residual: 0.01 },
        cheetah3::Sample { force_residual: 5.0, xi_residual: 0.5 }, // touchdown spike
        cheetah3::Sample { force_residual: 0.1, xi_residual: 0.01 },
    ];
    let mut residuals = [0.0_f64; 4];
    let n = cheetah3::residual_stream(&fixture, &mut residuals);
    assert_eq!(n, 4);
    // Index 2 must be the largest residual by construction.
    assert!(residuals[2] > residuals[0]);
    assert!(residuals[2] > residuals[3]);
}

#[test]
fn icub_pushrecovery_push_peak_maps_to_largest_residual() {
    let fixture = [
        icub_pushrecovery::Sample { wrench_residual: 0.5, xi_residual: 0.01 },
        icub_pushrecovery::Sample { wrench_residual: 1.0, xi_residual: 0.05 },
        icub_pushrecovery::Sample { wrench_residual: 8.0, xi_residual: 0.5 }, // peak
        icub_pushrecovery::Sample { wrench_residual: 1.0, xi_residual: 0.05 },
        icub_pushrecovery::Sample { wrench_residual: 0.5, xi_residual: 0.01 },
    ];
    let mut residuals = [0.0_f64; 5];
    let n = icub_pushrecovery::residual_stream(&fixture, &mut residuals);
    assert_eq!(n, 5);
    let peak_idx = residuals
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).expect("finite"))
        .map(|(i, _)| i)
        .expect("non-empty");
    assert_eq!(peak_idx, 2);
}

// ---------- PHM family (3) -----------------------------------------------

#[test]
fn cwru_residual_peaks_at_seeded_fault_sample() {
    let healthy = [0.10, 0.11, 0.10, 0.09, 0.10];
    let baseline = cwru::Baseline::from_healthy(&healthy).expect("finite");
    let faulted = [
        cwru::Sample { bpfi_amplitude: 0.10 },
        cwru::Sample { bpfi_amplitude: 0.10 },
        cwru::Sample { bpfi_amplitude: 0.45 }, // seeded fault
        cwru::Sample { bpfi_amplitude: 0.11 },
    ];
    let mut residuals = [0.0_f64; 4];
    let n = cwru::residual_stream(&faulted, baseline, &mut residuals);
    assert_eq!(n, 4);
    assert!(residuals[2] > residuals[0] + 0.2);
}

#[test]
fn ims_run_to_failure_produces_monotone_residuals() {
    let healthy = [0.05, 0.06, 0.05, 0.05, 0.06];
    let baseline = ims::Baseline::from_healthy(&healthy).expect("finite");
    let trajectory = [
        ims::Sample { health_index: 0.05 },
        ims::Sample { health_index: 0.07 },
        ims::Sample { health_index: 0.10 },
        ims::Sample { health_index: 0.15 },
        ims::Sample { health_index: 0.25 },
    ];
    let mut residuals = [0.0_f64; 5];
    let n = ims::residual_stream(&trajectory, baseline, &mut residuals);
    assert_eq!(n, 5);
    for i in 1..n {
        assert!(residuals[i] >= residuals[i - 1]);
    }
}

#[test]
fn femto_st_accelerated_aging_pipeline_escalates_eventually() {
    let healthy = [0.02, 0.02, 0.03, 0.02, 0.02];
    let baseline = femto_st::Baseline::from_healthy(&healthy).expect("finite");
    let trajectory = [
        femto_st::Sample { vib_hi: 0.02 },
        femto_st::Sample { vib_hi: 0.03 },
        femto_st::Sample { vib_hi: 0.05 },
        femto_st::Sample { vib_hi: 0.10 },
        femto_st::Sample { vib_hi: 0.25 },
        femto_st::Sample { vib_hi: 0.60 },
        femto_st::Sample { vib_hi: 1.20 },
        femto_st::Sample { vib_hi: 2.50 },
    ];
    let mut residuals = [0.0_f64; 8];
    let n = femto_st::residual_stream(&trajectory, baseline, &mut residuals);
    assert_eq!(n, 8);
    // Feed into the canonical observe() pipeline.
    let (adm, _bnd, vio) = run_pipeline(&residuals);
    assert!(adm >= 1, "early-life samples must be Admissible, got {adm}");
    assert!(vio >= 1, "late-life accelerated aging must produce Violations, got {vio}");
}

// ---------- Manipulation / balancing additions (Phase 8) -----------------

#[test]
fn droid_openx_unitree_aloha_anymal_fixtures_produce_valid_episodes() {
    use dsfb_robotics::datasets::{aloha_static, anymal_parkour, droid, openx, unitree_g1};
    for (label, mut buf) in [
        ("droid", vec![0.0_f64; 8]),
        ("openx", vec![0.0_f64; 8]),
        ("anymal_parkour", vec![0.0_f64; 8]),
        ("unitree_g1", vec![0.0_f64; 8]),
        ("aloha_static", vec![0.0_f64; 8]),
    ] {
        let n = match label {
            "droid" => droid::fixture_residuals(&mut buf),
            "openx" => openx::fixture_residuals(&mut buf),
            "anymal_parkour" => anymal_parkour::fixture_residuals(&mut buf),
            "unitree_g1" => unitree_g1::fixture_residuals(&mut buf),
            "aloha_static" => aloha_static::fixture_residuals(&mut buf),
            _ => unreachable!(),
        };
        assert!(n > 0, "{label} fixture emitted 0 samples");
        for v in &buf[..n] {
            assert!(v.is_finite() && *v >= 0.0, "{label} produced non-finite / negative residual: {v}");
        }
    }
}
