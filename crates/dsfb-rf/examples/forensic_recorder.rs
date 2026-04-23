//! Forensic Recorder — Post-Mortem Physics Report Generator
//!
//! ## Purpose
//!
//! Replays a recorded (or synthetic) IQ-residual tape through the full
//! DSFB-RF pipeline and emits a structured **Post-Mortem Physics Report**
//! to stdout (text format; optionally JSON with `--features serde`).
//!
//! The report covers:
//!
//! | Section | Content |
//! |---------|---------|
//! | §1 Tape Summary | Sample count, carrier, bandwidth, temperature |
//! | §2 Grammar Timeline | Grammar state per window with Landauer class |
//! | §3 Thermodynamic Budget | Structural energy waste (J/window, W) |
//! | §4 Fisher-Rao Trajectory | Geodesic distances between distribution epochs |
//! | §5 Quantum Noise Floor | R_QT, regime, SQL margin |
//! | §6 TDA Persistence Events | Birth/death pairs, topological innovation score |
//! | §7 RG Flow Classification | EnvironmentHypothesis — LocalFluke vs Systemic |
//! | §8 Swarm Consensus | N-node BFT modal grammar, quorum fraction |
//! | §9 Physics Mechanisms | Candidate physical failure modes per episode |
//!
//! ## Usage
//!
//! ```text
//! cargo run --features std --example forensic_recorder
//! cargo run --features std --example forensic_recorder -- --scenario jammer_onset
//! cargo run --features std --example forensic_recorder -- --scenario clock_drift
//! ```
//!
//! ## Synthetic Scenario Design
//!
//! This example uses *fully synthetic* residual sequences that are
//! structurally representative of the described physical conditions.
//! No real-world capture data is required or assumed. The sequences
//! are parameterised by `lcg_step` for determinism.
//!
//! ## Non-Claim
//!
//! The Landauer energy costs reported are the *minimum thermodynamic costs
//! implied by the observations*, not the actual power dissipation of the
//! receiver hardware.  The quantum noise floor analysis (§5) applies only
//! to receivers operating in the QuantumLimited regime (cryogenic/Rydberg);
//! all commercial SDRs are in the DeepThermal regime.

#[cfg(feature = "std")]
fn main() {
    use dsfb_rf::{DsfbRfEngine};
    use dsfb_rf::platform::PlatformContext;
    use dsfb_rf::grammar::{GrammarState, ReasonCode};
    use dsfb_rf::impairment::lcg_step;
    use dsfb_rf::tda::{detect_topological_innovation, PersistenceEvent};
    use dsfb_rf::energy_cost::{landauer_audit, LandauerClass};
    use dsfb_rf::fisher_geometry::{GaussPoint, ManifoldTracker};
    use dsfb_rf::quantum_noise::{QuantumNoiseTwin, ReceiverRegime};
    use dsfb_rf::rg_flow::{compute_rg_flow, RgFlowClass};
    use dsfb_rf::swarm_consensus::{GrammarVote, compute_consensus};
    use dsfb_rf::math::{mean_f32, std_dev_f32};

    extern crate std;
    use std::{println, vec::Vec, string::String};

    // ── Command-line scenario selection ──────────────────────────────────
    let args: Vec<String> = std::env::args().collect();
    let scenario = args.windows(2)
        .find(|w| w[0] == "--scenario")
        .map(|w| w[1].as_str())
        .unwrap_or("jammer_onset");

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     DSFB-RF  POST-MORTEM PHYSICS REPORT                     ║");
    println!("║     Forensic Recorder v2.0 — Invariant Forge LLC            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Scenario: {}", scenario);

    // ── §1 Tape Parameters ────────────────────────────────────────────────
    let carrier_hz      = 915.0e6_f32;   // 915 MHz ISM
    let bandwidth_hz    = 200.0e3_f32;   // 200 kHz channel
    let fs_hz           = 1.0e6_f32;     // 1 MHz sample rate
    let temp_k          = 290.0_f32;     // ambient 290 K (~17 °C)
    let rho_nominal     = 0.10_f32;      // nominal envelope radius
    let n_windows: usize = 20;           // total analysis windows
    let window_size: usize = 64;         // samples per window

    println!("§1  TAPE SUMMARY");
    println!("    Carrier:    {:.3} MHz", carrier_hz / 1e6);
    println!("    Bandwidth:  {:.1} kHz", bandwidth_hz / 1e3);
    println!("    Fs:         {:.3} MHz", fs_hz / 1e6);
    println!("    Temperature:{:.1} K  ({:.1} °C)", temp_k, temp_k - 273.15);
    println!("    Windows:    {}  ×  {} samples", n_windows, window_size);
    println!();

    // ── §2 Grammar Timeline ───────────────────────────────────────────────
    println!("§2  GRAMMAR TIMELINE");
    println!("    {:>4}  {:>10}  {:>12}  {:>10}  {:>10}",
        "Win#", "GrammarState", "LandauerClass", "EnergyJ", "PowerW");

    let mut engine = DsfbRfEngine::<16, 4, 16>::new(rho_nominal, 2.0_f32);
    let ctx = PlatformContext::operational();

    // Generate residual sequences per scenario
    let mut lcg_state: u32 = 0xDEAD_BEEFu32;
    let mut all_grammar: Vec<GrammarState> = Vec::new();
    let mut all_landauer: Vec<dsfb_rf::energy_cost::LandauerAudit> = Vec::new();
    let mut manifold_tracker = ManifoldTracker::new();

    for win in 0..n_windows {
        let mut residuals = [0.0_f32; 64];

        // Generate window residuals
        for r in residuals.iter_mut() {
            lcg_state = lcg_step(lcg_state);
            let noise = (lcg_state as f32 / u32::MAX as f32) * 2.0 - 1.0;
            let amplitude = match scenario {
                "jammer_onset" => {
                    if win < 8 {
                        0.04_f32 // nominal
                    } else if win < 14 {
                        0.04 + 0.20 * (win - 8) as f32 / 6.0 // ramp-up
                    } else {
                        0.24 // sustained violation
                    }
                }
                "clock_drift" => {
                    // Slow quadratic drift — TCXO warmup profile
                    0.04 + 0.001 * (win as f32 * win as f32).min(0.15)
                }
                _ => 0.04_f32, // "quiet" baseline
            };
            *r = noise * amplitude;
        }

        // Observe all samples in the window
        let mut last_grammar = GrammarState::Admissible;
        for &sample in residuals.iter() {
            let result = engine.observe(sample.abs(), ctx);
            last_grammar = result.grammar;
        }
        all_grammar.push(last_grammar);

        // Compute Landauer audit for this window
        let obs_sigma_sq = std_dev_f32(&residuals).powi(2).max(1e-30);
        let audit = landauer_audit(obs_sigma_sq, bandwidth_hz, temp_k, fs_hz);
        all_landauer.push(audit);

        // Fisher-Rao tracker: track distribution (mu=0, sigma=sqrt(obs_sigma_sq))
        let mu = mean_f32(&residuals);
        let sigma = std_dev_f32(&residuals).max(1e-6);
        let gp = GaussPoint { mu, sigma };
        let _ = manifold_tracker.push(gp);

        let gs_str = match last_grammar {
            GrammarState::Admissible => "Admissible ",
            GrammarState::Boundary(_) => "Boundary   ",
            GrammarState::Violation  => "Violation  ",
        };
        let lc_str = match audit.class {
            LandauerClass::SubThermal    => "Sub‑Thermal ",
            LandauerClass::Thermal       => "Thermal     ",
            LandauerClass::MildBurden    => "MildBurden  ",
            LandauerClass::ModerateBurden => "ModerateBrd ",
            LandauerClass::SevereBurden  => "SevereBurden",
        };
        println!("    {:>4}  {}  {}  {:.3e}  {:.3e}",
            win, gs_str, lc_str, audit.energy_joules, audit.power_watts);
    }
    println!();

    // ── §3 Thermodynamic Budget ───────────────────────────────────────────
    println!("§3  THERMODYNAMIC BUDGET (Landauer)");
    let total_j: f32 = all_landauer.iter().map(|a| a.energy_joules).sum();
    let peak_w:  f32 = all_landauer.iter().map(|a| a.power_watts).fold(0.0_f32, f32::max);
    let n_burd  = all_landauer.iter().filter(|a| {
        matches!(a.class, LandauerClass::ModerateBurden | LandauerClass::SevereBurden)
    }).count();
    println!("    Total structural energy: {:.3e} J", total_j);
    println!("    Peak structural power:   {:.3e} W", peak_w);
    println!("    High-burden windows:     {} / {}", n_burd, n_windows);
    println!("    [NON-CLAIM: These are minimum thermodynamic costs implied by the");
    println!("    observations, not hardware dissipation — Landauer (1961)]");
    println!();

    // ── §4 Fisher-Rao Trajectory ──────────────────────────────────────────
    println!("§4  FISHER-RAO MANIFOLD TRAJECTORY");
    println!("    Cumulative geodesic path length: {:.4}", manifold_tracker.cumulative_length());
    println!("    Peak single-step distance:       {:.4}", manifold_tracker.peak_distance());
    let drift_label = if manifold_tracker.cumulative_length() > 2.0 {
        "NonLinear/Oscillatory (escalated drift)"
    } else if manifold_tracker.cumulative_length() > 0.5 {
        "Settling (moderate drift)"
    } else {
        "Linear (minimal drift)"
    };
    println!("    Drift characterisation: {}", drift_label);
    println!("    [Fisher-Rao metric: Atkinson & Mitchell (1981), Calvo & Oller (1990)]");
    println!();

    // ── §5 Quantum Noise Floor ────────────────────────────────────────────
    println!("§5  QUANTUM NOISE FLOOR ANALYSIS");
    let qt = QuantumNoiseTwin::new(carrier_hz, bandwidth_hz, temp_k, 0.0);
    let regime_str = match qt.regime {
        ReceiverRegime::DeepThermal        => "DeepThermal (all commercial SDRs)",
        ReceiverRegime::TransitionRegime   => "TransitionRegime",
        ReceiverRegime::QuantumLimited     => "QuantumLimited (requires cryogenics)",
        ReceiverRegime::BelowSQL           => "BelowSQL (squeezed-light receiver)",
    };
    println!("    Carrier:              {:.3} MHz", carrier_hz / 1e6);
    println!("    R_QT (ħω/kT):        {:.3e}", qt.r_qt);
    println!("    Shot noise floor:     {:.3e} W", qt.shot_noise_w);
    println!("    Thermal noise floor:  {:.3e} W", qt.thermal_noise_w);
    println!("    Effective floor σ²:   {:.3e}", qt.sigma_sq_floor());
    println!("    SQL margin:           {:.3} dB", qt.sql_margin());
    println!("    Receiver regime:      {}", regime_str);
    println!("    [NON-CLAIM: QuantumLimited operation requires millikelvin cryogenics]");
    println!();

    // ── §6 TDA Persistence Events ─────────────────────────────────────────
    println!("§6  TDA PERSISTENCE EVENTS");
    // Build residual window from the highest-energy segment for TDA
    let peak_win = all_landauer.iter()
        .enumerate()
        .max_by(|a, b| a.1.energy_joules.partial_cmp(&b.1.energy_joules).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(n_windows - 1);

    // Re-generate that window's residuals for TDA
    let mut tda_residuals = [0.0_f32; 64];
    let mut state2: u32 = 0xDEAD_BEEFu32;
    for w in 0..=peak_win {
        for r in tda_residuals.iter_mut() {
            state2 = lcg_step(state2);
            let noise = (state2 as f32 / u32::MAX as f32) * 2.0 - 1.0;
            let amplitude = match scenario {
                "jammer_onset" => {
                    if w < 8 { 0.04 }
                    else if w < 14 { 0.04 + 0.20 * (w - 8) as f32 / 6.0 }
                    else { 0.24 }
                }
                "clock_drift" => 0.04 + 0.001 * (w as f32 * w as f32).min(0.15),
                _ => 0.04,
            };
            *r = noise * amplitude;
        }
    }

    let norms: [f32; 64] = tda_residuals.map(|x| x.abs());
    let tda_result = detect_topological_innovation(&norms, 0.05);
    println!("    Peak window (win #{}):", peak_win);
    if let Some(state) = tda_result {
        println!("    Betti\u{2080}:               {}", state.betti0);
        println!("    Innovation score:     {:.3}", state.innovation_score);
        println!("    Total persistence:    {:.4}", state.total_persistence);
        println!("    Birth events:         {}", state.n_births);
        println!("    Death events:         {}", state.n_deaths);
    } else {
        println!("    (Too few samples for TDA)");
    }
    println!();

    // ── §7 RG Flow Classification ─────────────────────────────────────────
    println!("§7  RG FLOW CLASSIFICATION");
    let events_slice = {
        // Synthesise persistence proxies from the Landauer energy profile.
        // In production, these would be drawn from the internal PersistenceLog;
        // here we use obs_sigma as a physics-grounded lifetime proxy.
        let mut synth = [PersistenceEvent { birth_radius: 0.0, death_radius: 0.0 }; 32];
        for (i, audit) in all_landauer.iter().enumerate().take(32) {
            let lt = dsfb_rf::math::sqrt_f32(audit.obs_sigma_sq.max(1e-12));
            synth[i] = PersistenceEvent { birth_radius: 0.0, death_radius: lt };
        }
        synth
    };
    let n_events = n_windows.min(32);
    let eps0      = 0.005_f32;
    let delta_eps = 0.003_f32;
    let rg = compute_rg_flow(&events_slice, n_events, eps0, delta_eps);
    let rg_label = rg.class.label();
    println!("    RG class:             {}", rg_label);
    println!("    β_RG (decay exp):     {:.3}", rg.beta_rg);
    if let Some(s) = rg.stable_at {
        println!("    Stable at scale ε:    {:.4}", s);
    } else {
        println!("    Stable at scale ε:    (transient — no stability found)");
    }
    println!("    Warrants escalation:  {}", rg.class.warrants_boundary_escalation());
    println!("    [RG coarse-graining: Wilson & Kogut (1974), Edelsbrunner & Harer (2010)]");
    println!();

    // ── §8 Swarm Consensus ────────────────────────────────────────────────
    println!("§8  SWARM CONSENSUS (N=5 nodes, f=1 Byzantine)");
    // Construct 5 synthetic votes: 3 authentic (agree on current grammar),
    // 1 authentic (slightly different), 1 Byzantine (disagree)
    let modal_state = {
        let n_viol = all_grammar.iter().filter(|g| matches!(g, GrammarState::Violation)).count();
        let n_bdry = all_grammar.iter().filter(|g| matches!(g, GrammarState::Boundary(_))).count();
        if n_viol >= n_windows / 2 { GrammarState::Violation }
        else if n_bdry >= n_windows / 4 {
            GrammarState::Boundary(ReasonCode::SustainedOutwardDrift)
        } else { GrammarState::Admissible }
    };

    let votes = [
        GrammarVote { node_id: 1, state: modal_state, dsa_score: 0.72, episode_count: 87, hardware_authenticated: true },
        GrammarVote { node_id: 2, state: modal_state, dsa_score: 0.75, episode_count: 91, hardware_authenticated: true },
        GrammarVote { node_id: 3, state: modal_state, dsa_score: 0.68, episode_count: 83, hardware_authenticated: true },
        GrammarVote { node_id: 4, state: modal_state, dsa_score: 0.71, episode_count: 85, hardware_authenticated: true },
        // Byzantine node: reports Violation regardless (hardware_authenticated=false → filtered)
        GrammarVote { node_id: 5, state: GrammarState::Violation, dsa_score: 0.99, episode_count: 9999, hardware_authenticated: false },
    ];

    let consensus = compute_consensus(&votes, 1, true);
    let cs_str = match consensus.modal_state {
        GrammarState::Admissible    => "Admissible",
        GrammarState::Boundary(_)  => "Boundary",
        GrammarState::Violation     => "Violation",
    };
    println!("    Admitted votes:       {}", consensus.votes_admitted);
    println!("    Quarantined (KS):     {}", consensus.votes_quarantined);
    println!("    Unauthenticated:      {}", consensus.votes_unauthenticated);
    println!("    Quorum reached:       {}", consensus.quorum_reached);
    println!("    P(Admissible):        {:.3}", consensus.p_admissible);
    println!("    P(Boundary):          {:.3}", consensus.p_boundary);
    println!("    P(Violation):         {:.3}", consensus.p_violation);
    println!("    Modal grammar state:  {}", cs_str);
    println!("    Consensus DSA score:  {:.4}", consensus.consensus_dsa_score);
    println!();

    // ── §9 Physics Mechanisms ─────────────────────────────────────────────
    println!("§9  CANDIDATE PHYSICS MECHANISMS");
    let n_violations = all_grammar.iter().filter(|g| matches!(g, GrammarState::Violation)).count();
    let n_boundaries = all_grammar.iter().filter(|g| matches!(g, GrammarState::Boundary(_))).count();
    println!("    Grammar summary: {} Admissible / {} Boundary / {} Violation",
        n_windows - n_violations - n_boundaries, n_boundaries, n_violations);
    if n_violations > 0 {
        println!("    → Structural violation detected — candidate mechanisms:");
        println!("      • CW tone injection (coherent interference onset)");
        println!("      • Narrowband jammer (frequency-stable elevated floor)");
        println!("      • PLL injection locking (carrier structure in residual)");
        if rg.class == RgFlowClass::SystemicEnvironmentChange {
            println!("      • Systemic environment change (RG-confirmed: β_RG={:.2})", rg.beta_rg);
        }
    } else if n_boundaries > n_windows / 4 {
        println!("    → Boundary-dominated — candidate mechanisms:");
        println!("      • Slow LO drift (TCXO warmup, Allan slope α≈-1)");
        println!("      • Propagation geometry change (geometry-induced drift)");
        println!("      • AGC overshoot transient (amplitude control settling)");
    } else {
        println!("    → Nominal operation — no structural anomaly detected.");
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  END OF POST-MORTEM PHYSICS REPORT                          ║");
    println!("║  All thermodynamic quantities are derived from observational ║");
    println!("║  residuals only.  No upstream receiver state was modified.   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

#[cfg(not(feature = "std"))]
fn main() {
    // no_std: nothing to run
}
