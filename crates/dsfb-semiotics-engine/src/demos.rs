//! Small deterministic demo traces reused by examples and tests.

use anyhow::Result;

use crate::engine::settings::{EngineSettings, SmoothingMode};
use crate::engine::types::{EnvelopeMode, GrammarState};
use crate::live::{OnlineStructuralEngine, Real};
use crate::math::envelope::EnvelopeSpec;

/// Runs the bounded failure-injection demo and returns the printed trace.
pub fn synthetic_failure_injection_trace() -> Result<String> {
    let mut settings = EngineSettings::default();
    settings.online.history_buffer_capacity = 48;

    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "synthetic_failure_injection",
        vec!["signal".to_string()],
        1.0,
        EnvelopeSpec {
            name: "synthetic_failure_envelope".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 0.95,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        settings,
    )?;

    let mut previous_syntax = String::new();
    let mut previous_grammar = GrammarState::Admissible;
    let mut previous_semantic = String::new();
    let mut lines = vec![
        "Synthetic failure injection trace".to_string(),
        "Nominal oscillation transitions into additive drift under a fixed envelope.".to_string(),
    ];

    for step in 0..90 {
        let time = step as f64;
        let nominal = 0.28 * (0.12 * time).sin();
        let drift = if step < 42 {
            0.0
        } else {
            0.012 * (step - 42) as f64
        };
        let value = nominal + drift;
        let status = engine.push_residual_sample(time, &[value as Real])?;

        if step == 0 || status.syntax_label != previous_syntax {
            lines.push(format!(
                "T+{time:.0}s: Syntax Change Detected -> {}",
                status.syntax_label
            ));
            previous_syntax = status.syntax_label.clone();
        }
        if step == 0 || status.grammar_state != previous_grammar {
            lines.push(format!(
                "T+{time:.0}s: Grammar State -> {:?} ({}, trust={:.3})",
                status.grammar_state, status.grammar_reason_text, status.trust_scalar
            ));
            previous_grammar = status.grammar_state;
        }
        if step == 0 || status.semantic_disposition != previous_semantic {
            let selected = if status.selected_heuristic_ids.is_empty() {
                "none".to_string()
            } else {
                status.selected_heuristic_ids.join(", ")
            };
            lines.push(format!(
                "T+{time:.0}s: Semantic Interpretation -> {} [{}]",
                status.semantic_disposition, selected
            ));
            previous_semantic = status.semantic_disposition.clone();
        }
    }

    Ok(lines.join("\n"))
}

/// Runs a physically grounded demo in which slew-rich vibration-like behavior transitions into a
/// slower drift-like regime with inherited signal units.
pub fn vibration_to_thermal_drift_trace() -> Result<String> {
    let mut settings = EngineSettings::default();
    settings.online.history_buffer_capacity = 64;
    settings.smoothing.mode = SmoothingMode::ExponentialMovingAverage;
    settings.smoothing.exponential_alpha = 0.28;

    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "vibration_to_thermal_drift",
        vec!["bearing_gap_mm".to_string()],
        1.0,
        EnvelopeSpec {
            name: "bearing_gap_envelope_mm".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 0.68,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        settings,
    )?;

    let mut previous_syntax = String::new();
    let mut previous_grammar = GrammarState::Admissible;
    let mut previous_semantic = String::new();
    let mut lines = vec![
        "Vibration to thermal drift trace".to_string(),
        "Residual units inherit from the source signal here: millimeters, millimeters/second, and millimeters/second^2.".to_string(),
        "High-frequency oscillation is followed by slower monotone outward drift under optional conservative smoothing.".to_string(),
    ];

    for step in 0..120 {
        let time = step as f64;
        let vibration = if step < 52 {
            0.18 * (0.55 * time).sin() + 0.04 * (1.15 * time).sin()
        } else {
            0.04 * (0.30 * time).sin()
        };
        let thermal_drift = if step < 52 {
            0.0
        } else {
            0.0075 * (step - 52) as f64
        };
        let value_mm = vibration + thermal_drift;
        let status = engine.push_residual_sample(time, &[value_mm as Real])?;

        if step == 0 || status.syntax_label != previous_syntax {
            lines.push(format!(
                "T+{time:.0}s: Syntax -> {} | residual={:.3} mm | drift={:.3} mm/s | slew={:.3} mm/s^2",
                status.syntax_label, status.residual_norm, status.drift_norm, status.slew_norm
            ));
            previous_syntax = status.syntax_label.clone();
        }
        if step == 0 || status.grammar_state != previous_grammar {
            lines.push(format!(
                "T+{time:.0}s: Grammar -> {:?} ({}, trust={:.3})",
                status.grammar_state, status.grammar_reason_text, status.trust_scalar
            ));
            previous_grammar = status.grammar_state;
        }
        if step == 0 || status.semantic_disposition != previous_semantic {
            let selected = if status.selected_heuristic_ids.is_empty() {
                "none".to_string()
            } else {
                status.selected_heuristic_ids.join(", ")
            };
            lines.push(format!(
                "T+{time:.0}s: Semantic Interpretation -> {} [{}]",
                status.semantic_disposition, selected
            ));
            previous_semantic = status.semantic_disposition.clone();
        }
    }

    Ok(lines.join("\n"))
}
