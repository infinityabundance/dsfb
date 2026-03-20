use anyhow::Result;
use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::types::{EnvelopeMode, GrammarState};
use dsfb_semiotics_engine::live::{OnlineStructuralEngine, Real};
use dsfb_semiotics_engine::math::envelope::EnvelopeSpec;

fn main() -> Result<()> {
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

    println!("Synthetic failure injection trace");
    println!("Nominal oscillation transitions into additive drift under a fixed envelope.");

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
            println!(
                "T+{time:.0}s: Syntax Change Detected -> {}",
                status.syntax_label
            );
            previous_syntax = status.syntax_label.clone();
        }
        if step == 0 || status.grammar_state != previous_grammar {
            println!(
                "T+{time:.0}s: Grammar State -> {:?} ({})",
                status.grammar_state, status.grammar_reason_text
            );
            previous_grammar = status.grammar_state;
        }
        if step == 0 || status.semantic_disposition != previous_semantic {
            let selected = if status.selected_heuristic_ids.is_empty() {
                "none".to_string()
            } else {
                status.selected_heuristic_ids.join(", ")
            };
            println!(
                "T+{time:.0}s: Semantic Interpretation -> {} [{}]",
                status.semantic_disposition, selected
            );
            previous_semantic = status.semantic_disposition.clone();
        }
    }

    Ok(())
}
