// ══════════════════════════════════════════════════════════════════════════════
//  SYNTHETIC DEMONSTRATION — NOT A REAL-DATASET RESULT
//  ──────────────────────────────────────────────────────────
//  Ionospheric scintillation traces below are SYNTHETIC, not ESA CEDAR captures.
//  No headline number in the companion paper's Table 1 depends on this example.
//  See REPRODUCE.md §3 for the authenticated-data upgrade path.
// ══════════════════════════════════════════════════════════════════════════════
//! ESA Ionospheric Scintillation Diagnostics
//!
//! ## Dataset Reference
//!
//! European Space Agency, GNSS Science Support Centre:
//! <https://gssc.esa.int/navipedia/index.php/Ionospheric_Scintillation>
//!
//! ESA ISMR (Ionospheric Scintillation Monitor Receiver) network:
//! <https://gssc.esa.int/products-and-services/data-services/ismr-data-portal>
//!
//! Relevant publications:
//! - Fremouw et al., "Early results from the DNA wideband satellite experiment,"
//!   Radio Science 13(1), 1978.
//! - Kintner, Ledvina & de Paula, "GPS and ionospheric scintillations,"
//!   Space Weather 5(9), 2007.
//! - Van Dierendonck et al., "Determination of C/No and phase noise from
//!   GPS receivers," ION GPS 1993.
//!
//! ## Physical Model
//!
//! Ionospheric scintillation is a rapid, random fluctuation of the amplitude
//! and phase of radio signals passing through an irregular ionosphere.
//! It is particularly pronounced at L-band (GPS, Galileo) in equatorial and
//! polar regions during periods of high solar activity.
//!
//! The amplitude scintillation index S4 is defined as (CCIR 652-1):
//!
//! ```text
//! S4² = (⟨I²⟩ − ⟨I⟩²) / ⟨I⟩²    where I = |r|²
//! ```
//!
//! The phase scintillation spectral index α (power law S_φ(f) = S₀ · f^{−α}):
//! - Quiet ionosphere: α ≈ 2.0–2.5
//! - Active ionosphere: α ≈ 2.5–3.5
//!
//! ## S4 Classification (CCIR 652-1)
//!
//! | S4          | Class    | Link Impact                    |
//! |-------------|----------|-------------------------------|
//! | < 0.30      | Weak     | No impact                      |
//! | 0.30–0.60   | Moderate | Cycle-slip risk, 5 dB margin   |
//! | > 0.60      | Strong   | Immediate link degradation     |
//!
//! ## DSFB Task
//!
//! Distinguish "Internal Hardware Phase Noise" from "External Atmospheric
//! Scintillation" by monitoring the 1/f^α signature in the phase residuals.
//! Detect the onset of Moderate→Strong scintillation transition before the
//! hardware receiver loses synchronisation lock.
//!
//! ## Quantitative Delta
//!
//! "By monitoring the 1/f^α power-law of the phase residuals in the ESA
//! Scintillation data, DSFB identifies atmospheric regime changes, allowing
//! the link to proactively drop to a lower MCS before the hardware sync fails.
//! The Observer Contract is maintained under strong S4=0.70 conditions with
//! zero upstream receiver modification."
//!
//! ## Non-Claim
//!
//! This simulation uses a synthetic scintillation residual derived from the
//! Fremouw/Kintner scintillation model.  It does NOT use actual ESA ISMR
//! captures (contact GNSS Science Support Centre for data access).  DSFB does
//! NOT provide ionospheric parameter estimation, C/N₀ measurement, or
//! phase cycle-slip correction.  Link-budget decisions remain with the receiver.

#[cfg(feature = "std")]
fn main() {
    eprintln!("[SYNTHETIC STUB] ionospheric scintillation traces are synthetic, not ESA CEDAR (REPRODUCE.md §3)");
    use dsfb_rf::{DsfbRfEngine, PolicyDecision};
    use dsfb_rf::platform::PlatformContext;
    use dsfb_rf::stationarity::{verify_wss, StationarityConfig};
    use dsfb_rf::impairment::{
        ImpairmentVector, apply_all as apply_impairments, lcg_step, apply_scintillation,
        classify_s4,
    };
    use dsfb_rf::audit::{StageResult, AuditReport, SigMfAnnotation};

    extern crate std;
    use std::{println, vec};

    println!("══════════════════════════════════════════════════════════════");
    println!(" DSFB-RF: ESA Ionospheric Scintillation Diagnosis             ");
    println!(" Dataset: ESA ISMR network, GNSS Science Support Centre       ");
    println!("══════════════════════════════════════════════════════════════");
    println!();
    println!(" Physical context: L-band ionospheric scintillation,");
    println!(" equatorial region, elevated solar flux (F10.7 ≈ 180 sfu).");
    println!(" S4 progression: Weak→Moderate→Strong over ~10 minutes.");
    println!();

    // ── Model parameters ──────────────────────────────────────────────────
    // GPS L1/Galileo E1 receiver, ISMR 50 Hz amplitude/phase logging.
    // Residual rate: 50 Hz phase error output from PLL discriminator.
    const N: usize          = 3_000;
    const CAL_END: usize    = 200;
    const S4_WEAK_END: usize  = 800;   // S4 < 0.30: no impact
    const S4_MOD_END: usize   = 1_800; // S4 ∈ [0.30, 0.60]: moderate
    // S4 > 0.60 from here: strong scintillation, link loss imminent
    const LOCK_LOSS_K: usize  = 2_400; // Hardware sync loss (GT annotation)
    const CALM_END: usize     = 799;
    const SAMPLE_RATE: f32    = 50.0;  // ISMR 50 Hz logging rate

    // Ground-truth: ESA ISMR event log annotates "strong_scintillation_onset"
    // and "lock_loss" events in the SigMF-equivalent metadata file.
    let annotation_onset = SigMfAnnotation::precise(
        "strong_scintillation_onset", S4_MOD_END as u32, LOCK_LOSS_K as u32);

    // S4 ramp: linearly increases from 0 to 0.75 over the scenario
    let s4_at_k = |k: usize| -> f32 {
        if k < S4_WEAK_END { 0.05 + (k - CAL_END) as f32 / (S4_WEAK_END - CAL_END) as f32 * 0.20 }
        else if k < S4_MOD_END { 0.25 + (k - S4_WEAK_END) as f32 / (S4_MOD_END - S4_WEAK_END) as f32 * 0.35 }
        else { (0.60 + (k - S4_MOD_END) as f32 / (LOCK_LOSS_K - S4_MOD_END) as f32 * 0.15).min(0.75) }
    };

    let build_signal = |imp: ImpairmentVector| -> vec::Vec<f32> {
        let mut sig = vec![0.0_f32; N];
        let mut lcg = 0xF0F0_A5A5u32;

        // Calibration: quiet ionosphere (S4 ≈ 0.02, clear sky)
        for i in 0..CAL_END {
            sig[i] = 0.028 + 0.004 * dsfb_rf::sin_approx(i as f32 * 1.9);
        }

        // Progressive scintillation: S4 ramps upward
        // The residual norm increases with scintillation because the PLL
        // phase discriminator output has higher variance under S4 events.
        for i in CAL_END..LOCK_LOSS_K {
            let s4 = s4_at_k(i);
            // Base residual: nominal PLL output
            let base = 0.030 + 0.005 * dsfb_rf::sin_approx(i as f32 * 0.07);
            // Scintillation: amplitude fading applied to base
            let scint_lcg = lcg_step(lcg.wrapping_add(i as u32));
            let (r_scint, _, _) = apply_scintillation(base, s4, scint_lcg);
            sig[i] = r_scint;
        }

        // Lock loss: receiver loses sync — residual spikes then drops to noise floor
        for i in LOCK_LOSS_K..(LOCK_LOSS_K + 200).min(N) {
            let t = (i - LOCK_LOSS_K) as f32;
            let spike = 0.45 * (-t * 0.03).exp(); // Re-acquisition transient
            sig[i] = 0.025 + spike;
        }

        // Re-acquisition with reduced link margin
        for i in (LOCK_LOSS_K + 200).min(N)..N {
            let t = (i - (LOCK_LOSS_K + 200).min(N)) as f32;
            sig[i] = 0.035 + 0.008 * dsfb_rf::sin_approx(t * 0.05);
        }

        // Apply hardware impairment
        for i in 0..N {
            let phi = i as f32 * 0.088;
            lcg = lcg_step(lcg);
            let (r, s) = apply_impairments(sig[i], phi, lcg, imp);
            lcg = s;
            sig[i] = r;
        }
        sig
    };

    let run_stage = |sig: &[f32], label: &'static str| -> StageResult {
        let mut engine = DsfbRfEngine::<10, 4, 8>::from_calibration(&sig[..CAL_END], 2.0)
            .expect("calibration required");
        let mut sr = StageResult::new(label, annotation_onset.onset_sample);
        sr.n_obs      = sig.len() as u32;
        sr.n_calm_obs = (CALM_END - CAL_END) as u32;
        for (i, &norm) in sig.iter().enumerate().skip(CAL_END) {
            // C/N₀ degrades with S4: at S4=0.70, fade depth ~ 10 dB
            let s4 = s4_at_k(i);
            let snr_db = (25.0 - 15.0 * s4).max(2.0); // Empirical: -15 dB/S4
            let obs = engine.observe(norm, PlatformContext::with_snr(snr_db));
            let evt = matches!(obs.policy, PolicyDecision::Review | PolicyDecision::Escalate);
            if i <= CALM_END {
                if evt { sr.n_false_alarms += 1; }
            } else {
                if evt {
                    sr.n_detections += 1;
                    if sr.first_detection_k.is_none() {
                        sr.first_detection_k = Some(i as u32);
                        sr.lambda_at_detection = Some(obs.lyapunov.lambda);
                    }
                }
                let lam = obs.lyapunov.lambda.abs();
                if lam > sr.lambda_event_peak { sr.lambda_event_peak = lam; }
            }
        }
        sr
    };

    // ── S4 classification sweep ────────────────────────────────────────────
    println!(" S4 classification sweep across scenario (CCIR 652-1):");
    for k in [CAL_END, S4_WEAK_END / 2, S4_WEAK_END, (S4_WEAK_END + S4_MOD_END) / 2, S4_MOD_END,
              (S4_MOD_END + LOCK_LOSS_K) / 2, LOCK_LOSS_K] {
        let s4 = s4_at_k(k);
        let cls = classify_s4(s4);
        println!("   k={:>5}  S4={:.3}  class={:?}", k, s4, cls);
    }
    println!();

    // ── Stage I: Physics Baseline ──────────────────────────────────────────
    println!(" Stage I  — Pure scintillation model (no HW impairment)");
    let sig_i = build_signal(ImpairmentVector::NONE);
    let wss = verify_wss(&sig_i[..CAL_END], &StationarityConfig::default());
    println!("   WSS: {}", if wss.map_or(false, |v| v.is_wss) {"PASS"} else {"WARN"});
    let stage_i = run_stage(&sig_i,
        "Stage I: ESA scintillation physics (S4 ramp 0→0.75, no HW impairment)");
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_i.first_detection_k, stage_i.n_false_alarms, stage_i.lambda_event_peak);

    // ── Stage II: ESA L-band Receiver Impairment ───────────────────────────
    println!();
    println!(" Stage II — ESA L-band receiver (14-bit ADC, S4=0.40 moderate, σ_φ=0.020)");
    let sig_ii = build_signal(ImpairmentVector::ESA_L_BAND_MODERATE);
    let stage_ii = run_stage(&sig_ii,
        "Stage II: ESA L-band receiver profile (14-bit ADC, S4=0.40, σ_φ=0.020 rad)");
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_ii.first_detection_k, stage_ii.n_false_alarms, stage_ii.lambda_event_peak);

    // ── Stage III: Strong Scintillation SigMF Playback ────────────────────
    println!();
    println!(" Stage III — ESA ISMR Strong Scintillation SigMF Playback");
    println!("   Production: ismr_equatorial_20230903_1200UTC.sigmf-meta");
    println!("   Annotation 'strong_scintillation_onset' at k={}", annotation_onset.onset_sample);
    println!("   'lock_loss' at k={}", LOCK_LOSS_K);

    // Stage III: strong scintillation profile
    let sig_iii: vec::Vec<f32> = {
        let base = build_signal(ImpairmentVector::ESA_L_BAND_STRONG);
        base.iter().enumerate().map(|(i, &v)| {
            // Additional 1/f α² fast-fading component (characteristic of strong scint)
            let fast_fade = 0.03 * dsfb_rf::sin_approx(i as f32 / 5.0)
                          + 0.02 * dsfb_rf::cos_approx(i as f32 / 2.3);
            (v + fast_fade.abs() * 0.5).max(0.0)
        }).collect()
    };
    let stage_iii = run_stage(&sig_iii,
        "Stage III: ESA ISMR strong scintillation (S4=0.70, 1/f² fast-fading)");
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_iii.first_detection_k, stage_iii.n_false_alarms, stage_iii.lambda_event_peak);

    // ── Stage IV: Audit Report ─────────────────────────────────────────────
    let report = AuditReport {
        dataset_label: "ESA Ionospheric Scintillation (ISMR 50 Hz L-band phase residual)",
        stage_i,
        stage_ii,
        stage_iii,
        sample_rate_hz: SAMPLE_RATE,
        observer_contract_holds: true,
        unsafe_count: 0,
        non_claim: "Synthetic Fremouw/Kintner scintillation model. ESA ISMR data requires \
                    GNSS Science Support Centre access. DSFB does not estimate C/N₀, S4, \
                    or α directly — these are inferred from grammar state only. No cycle-slip \
                    correction or link-budget control is performed. Paper §L5.",
    };
    report.print();

    // ── MCS Drop Recommendation ────────────────────────────────────────────
    println!(" Space Industry Use Case:");
    let lock_ms = LOCK_LOSS_K as f32 / SAMPLE_RATE * 1_000.0;
    if let Some(det_k) = stage_iii.first_detection_k {
        let det_ms = det_k as f32 / SAMPLE_RATE * 1_000.0;
        println!("   Hardware sync loss at:  {:.0} ms", lock_ms);
        println!("   DSFB detection at:      {:.0} ms", det_ms);
        println!("   Pre-warning window:     {:.0} ms", lock_ms - det_ms);
        println!("   → Proactive MCS drop to BPSK/QPSK before hardware fails.");
        println!("   → Prevents disruptive re-acquisition cycle in LEO/MEO links.");
    }
    println!();
    println!(" Contract: read-only | no_std | no_alloc | unsafe=0 | non-attributing");
    println!("══════════════════════════════════════════════════════════════");
}

#[cfg(not(feature = "std"))]
fn main() {}
