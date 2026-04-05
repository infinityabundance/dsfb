//! Tests for the minimal inline fast-path proxy (src/fast_path.rs).
//!
//! These tests verify correctness of the CPU reference implementation.
//! GPU tests are included where a wgpu adapter is available; they are skipped
//! gracefully otherwise.

use dsfb_computer_graphics::fast_path::{
    run_fast_path_cpu, try_run_fast_path_gpu, FAST_PATH_K, FAST_PATH_LAMBDA,
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn uniform_rgb(w: usize, h: usize, r: f32, g: f32, b: f32) -> Vec<[f32; 3]> {
    vec![[r, g, b]; w * h]
}

fn zero_history(n: usize) -> Vec<f32> {
    vec![0.0f32; n]
}

// ─── Determinism ──────────────────────────────────────────────────────────────

#[test]
fn fast_path_cpu_is_deterministic() {
    let (w, h) = (16, 16);
    let n = w * h;
    let cur = uniform_rgb(w, h, 0.6, 0.4, 0.3);
    let hist = uniform_rgb(w, h, 0.5, 0.5, 0.5);
    let res = zero_history(n);
    let drift = zero_history(n);

    let out_a =
        run_fast_path_cpu(&cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, false);
    let out_b =
        run_fast_path_cpu(&cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, false);

    assert_eq!(out_a.trust, out_b.trust, "trust must be deterministic");
    assert_eq!(
        out_a.residual_history_out, out_b.residual_history_out,
        "residual history must be deterministic"
    );
    assert_eq!(
        out_a.drift_history_out, out_b.drift_history_out,
        "drift history must be deterministic"
    );
}

// ─── Trust in [0, 1] ──────────────────────────────────────────────────────────

#[test]
fn fast_path_cpu_trust_always_in_unit_interval() {
    let (w, h) = (32, 32);
    let n = w * h;

    // Extreme residual case: current = white, history = black, no histories.
    let cur = uniform_rgb(w, h, 1.0, 1.0, 1.0);
    let hist = uniform_rgb(w, h, 0.0, 0.0, 0.0);
    let res = zero_history(n);
    let drift = zero_history(n);

    let out = run_fast_path_cpu(&cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, false);
    for (i, &t) in out.trust.iter().enumerate() {
        assert!(
            (0.0..=1.0).contains(&t),
            "trust[{i}] = {t} is outside [0, 1]"
        );
    }
}

#[test]
fn fast_path_cpu_local_agg_trust_always_in_unit_interval() {
    let (w, h) = (16, 16);
    let n = w * h;

    let cur = uniform_rgb(w, h, 0.9, 0.1, 0.5);
    let hist = uniform_rgb(w, h, 0.1, 0.9, 0.5);
    let res = vec![0.3f32; n];
    let drift = vec![-0.1f32; n];

    let out = run_fast_path_cpu(&cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, true);
    for (i, &t) in out.trust.iter().enumerate() {
        assert!(
            (0.0..=1.0).contains(&t),
            "local-agg trust[{i}] = {t} is outside [0, 1]"
        );
    }
}

// ─── Zero residual → max trust ────────────────────────────────────────────────

#[test]
fn fast_path_cpu_zero_residual_zero_history_yields_max_trust() {
    // If current == history, r_t = 0. With zero histories, d_t = 0, s_t = 0.
    // So u_t = 0, T_t = saturate(1 - k*0) = 1.
    let (w, h) = (8, 8);
    let n = w * h;
    let color = uniform_rgb(w, h, 0.5, 0.3, 0.7);
    let res = zero_history(n);
    let drift = zero_history(n);

    let out = run_fast_path_cpu(
        &color,
        &color, // history == current
        &res,
        &drift,
        w,
        h,
        FAST_PATH_LAMBDA,
        FAST_PATH_K,
        false,
    );

    for (i, &t) in out.trust.iter().enumerate() {
        assert!(
            (t - 1.0f32).abs() < 1e-6,
            "trust[{i}] should be 1.0 when residual=0, got {t}"
        );
    }
}

// ─── Constant residual → zero slew ────────────────────────────────────────────

#[test]
fn fast_path_cpu_constant_residual_produces_zero_slew() {
    // If the same residual repeats across frames, slew cancels out.
    // Frame 0: r_0 = R, d_0_in = 0, d_0 = R - 0 = R.
    // Frame 1: r_1 = R, d_1_in = R (from frame 0), d_1 = R - R = 0, s_1 = 0 - R = -R (non-zero).
    // Frame 2: r_2 = R, d_2_in = 0, d_2 = R - 0 = R... Actually slew stabilises over multiple frames.
    //
    // After several identical-residual frames, d becomes stable (0), so s becomes 0.
    // Verify that the slew component (drift_history_out) converges toward 0
    // after several passes with constant residual.

    let (w, h) = (4, 4);
    let n = w * h;
    let cur = uniform_rgb(w, h, 0.6, 0.6, 0.6);
    let hist = uniform_rgb(w, h, 0.5, 0.5, 0.5);
    // L1 residual per channel = |0.6 - 0.5| = 0.1; mean = 0.1.

    let mut res_h = zero_history(n);
    let mut drift_h = zero_history(n);

    // Run 6 frames with constant colour.
    for _ in 0..6 {
        let out = run_fast_path_cpu(
            &cur,
            &hist,
            &res_h,
            &drift_h,
            w,
            h,
            FAST_PATH_LAMBDA,
            FAST_PATH_K,
            false,
        );
        res_h = out.residual_history_out;
        drift_h = out.drift_history_out;
    }

    // After convergence, drift should be approximately 0 (constant residual → stable drift).
    for (i, &d) in drift_h.iter().enumerate() {
        assert!(
            d.abs() < 1e-5,
            "drift[{i}] = {d} should be ~0 after constant-residual convergence"
        );
    }
}

// ─── Local aggregation result equivalence to scalar for uniform inputs ────────

#[test]
fn fast_path_cpu_local_agg_equals_scalar_for_uniform_residual() {
    // When u is spatially uniform, the 3×3 mean equals the scalar value.
    let (w, h) = (8, 8);
    let n = w * h;
    let cur = uniform_rgb(w, h, 0.7, 0.4, 0.2);
    let hist = uniform_rgb(w, h, 0.5, 0.5, 0.5);
    let res = zero_history(n);
    let drift = zero_history(n);

    let out_no_agg =
        run_fast_path_cpu(&cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, false);
    let out_agg =
        run_fast_path_cpu(&cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, true);

    for i in 0..n {
        let diff = (out_no_agg.trust[i] - out_agg.trust[i]).abs();
        assert!(
            diff < 1e-5,
            "trust[{i}]: scalar={} agg={} diff={diff} — should match for uniform inputs",
            out_no_agg.trust[i],
            out_agg.trust[i]
        );
    }
}

// ─── History buffers update correctly ────────────────────────────────────────

#[test]
fn fast_path_cpu_history_buffers_update_to_r_t_and_d_t() {
    let (w, h) = (1, 1);
    let n = 1;
    // r_t = (|0.8-0.5| + |0.5-0.5| + |0.5-0.5|) / 3 = 0.3 / 3 = 0.1
    let cur = vec![[0.8f32, 0.5, 0.5]];
    let hist = vec![[0.5f32, 0.5, 0.5]];
    let res = zero_history(n); // r_{t-1} = 0
    let drift = zero_history(n); // d_{t-1} = 0

    let out = run_fast_path_cpu(&cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, false);

    let expected_r = 0.1f32;
    let expected_d = expected_r - 0.0;
    assert!((out.residual_history_out[0] - expected_r).abs() < 1e-6, "r_t mismatch");
    assert!((out.drift_history_out[0] - expected_d).abs() < 1e-6, "d_t mismatch");
}

// ─── GPU / CPU consistency ────────────────────────────────────────────────────

#[test]
fn fast_path_gpu_matches_cpu_within_tolerance_if_available() {
    let (w, h) = (8, 8);
    let n = w * h;
    let cur = uniform_rgb(w, h, 0.6, 0.4, 0.3);
    let hist = uniform_rgb(w, h, 0.5, 0.5, 0.5);
    let res = zero_history(n);
    let drift = zero_history(n);

    let cpu_out = run_fast_path_cpu(
        &cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, false,
    );

    match try_run_fast_path_gpu(
        &cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, false,
    ) {
        Ok(Some(gpu_out)) => {
            let tolerance = 1e-4f32;
            for i in 0..n {
                let diff = (cpu_out.trust[i] - gpu_out.trust[i]).abs();
                assert!(
                    diff <= tolerance,
                    "trust[{i}]: cpu={} gpu={} diff={diff} exceeds tolerance {tolerance}",
                    cpu_out.trust[i],
                    gpu_out.trust[i]
                );
            }
        }
        Ok(None) => {
            // No GPU adapter available; skip without failing.
            eprintln!("fast_path_gpu_matches_cpu: no wgpu adapter, test skipped");
        }
        Err(e) => panic!("GPU fast-path error: {e}"),
    }
}

// ─── No-allocation note verification ─────────────────────────────────────────

#[test]
fn fast_path_cpu_pre_allocates_before_inner_loop() {
    // This test exercises the CPU reference at a moderate resolution to confirm it
    // completes without panic. The implementation pre-allocates output Vecs before
    // the inner computation loop; this test is a smoke-check for that structure.
    let (w, h) = (64, 64);
    let n = w * h;
    let cur: Vec<[f32; 3]> = (0..n).map(|i| [i as f32 / n as f32, 0.3, 0.7]).collect();
    let hist = uniform_rgb(w, h, 0.4, 0.5, 0.6);
    let res = vec![0.05f32; n];
    let drift = vec![0.02f32; n];

    let out = run_fast_path_cpu(&cur, &hist, &res, &drift, w, h, FAST_PATH_LAMBDA, FAST_PATH_K, false);
    assert_eq!(out.trust.len(), n);
    assert_eq!(out.residual_history_out.len(), n);
    assert_eq!(out.drift_history_out.len(), n);
}
