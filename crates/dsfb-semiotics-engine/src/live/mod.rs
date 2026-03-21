//! Bounded-history live engine utilities for deployment-oriented deterministic replay and FFI use.
//!
//! The batch artifact pipeline remains free to materialize full histories for reports, while this
//! module keeps the online path memory-bounded through an explicit circular buffer.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::engine::bank::HeuristicBankRegistry;
use crate::engine::grammar_layer::evaluate_grammar_layer;
use crate::engine::semantics::{
    build_retrieval_index, retrieve_semantics_with_context, SemanticRetrievalContext,
    SemanticRetrievalIndex,
};
use crate::engine::settings::EngineSettings;
use crate::engine::sign_layer::construct_signs;
use crate::engine::syntax_layer::characterize_syntax_with_coordination_configured;
use crate::engine::types::{
    GrammarReasonCode, GrammarState, GrammarStatus, ResidualSample, ResidualTrajectory,
    SemanticDisposition, SemanticMatchResult, SyntaxCharacterization,
};
use crate::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use crate::math::envelope::{build_envelope, EnvelopeSpec};
use crate::math::metrics::euclidean_norm;
use crate::math::smoothing::smooth_residual_trajectory;

/// Stable machine-readable schema identifier for bounded online-engine status snapshots.
pub const LIVE_ENGINE_STATUS_SCHEMA_VERSION: &str = "dsfb-semiotics-live-status/v1";

/// Numeric type used by the bounded online engine path.
#[cfg(feature = "numeric-f32")]
pub type Real = f32;
/// Numeric type used by the bounded online engine path.
#[cfg(not(feature = "numeric-f32"))]
pub type Real = f64;

/// Returns the compile-time numeric mode used by the bounded online path.
#[must_use]
pub const fn numeric_mode_label() -> &'static str {
    if cfg!(feature = "numeric-f32") {
        "f32"
    } else {
        "f64"
    }
}

/// Fixed-capacity ring buffer with deterministic overwrite semantics.
#[derive(Clone, Debug)]
pub struct RingBuffer<T> {
    slots: Vec<Option<T>>,
    start: usize,
    len: usize,
}

impl<T> RingBuffer<T> {
    /// Creates an empty ring buffer with deterministic fixed capacity.
    pub fn new(capacity: usize) -> Result<Self> {
        if capacity == 0 {
            return Err(anyhow!("ring buffer capacity must be greater than zero"));
        }
        let mut slots = Vec::with_capacity(capacity);
        slots.resize_with(capacity, || None);
        Ok(Self {
            slots,
            start: 0,
            len: 0,
        })
    }

    /// Returns the fixed capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Returns the current number of retained entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the buffer currently retains no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Pushes one value, overwriting the oldest entry when the buffer is full.
    pub fn push(&mut self, value: T) -> Option<T> {
        if self.len < self.capacity() {
            let index = (self.start + self.len) % self.capacity();
            self.slots[index] = Some(value);
            self.len += 1;
            None
        } else {
            let index = self.start;
            self.start = (self.start + 1) % self.capacity();
            self.slots[index].replace(value)
        }
    }

    /// Returns the retained values in chronological order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).filter_map(|offset| {
            let index = (self.start + offset) % self.capacity();
            self.slots[index].as_ref()
        })
    }
}

/// Deterministic status snapshot emitted after each bounded online-engine update.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiveEngineStatus {
    pub schema_version: String,
    pub scenario_id: String,
    pub step: usize,
    pub time: f64,
    pub history_buffer_capacity: usize,
    pub current_history_len: usize,
    pub offline_history_len: Option<usize>,
    pub numeric_mode: String,
    pub residual_norm: f64,
    pub drift_norm: f64,
    pub slew_norm: f64,
    pub projection: [f64; 3],
    pub syntax_label: String,
    pub grammar_state: GrammarState,
    pub grammar_reason_code: GrammarReasonCode,
    pub grammar_reason_text: String,
    pub trust_scalar: f64,
    pub semantic_disposition_code: u8,
    pub semantic_disposition: String,
    pub selected_heuristic_ids: Vec<String>,
    pub note: String,
}

/// Bounded online engine that reuses the existing layered batch logic over a fixed trailing
/// window.
#[derive(Clone, Debug)]
pub struct OnlineStructuralEngine {
    scenario_id: String,
    channel_names: Vec<String>,
    dt: f64,
    envelope_spec: EnvelopeSpec,
    settings: EngineSettings,
    bank_registry: HeuristicBankRegistry,
    retrieval_index: SemanticRetrievalIndex,
    residual_history: RingBuffer<ResidualSample>,
    offline_history: Option<Vec<ResidualSample>>,
    next_step: usize,
}

impl OnlineStructuralEngine {
    /// Creates a bounded online engine with the builtin heuristic bank.
    pub fn with_builtin_bank(
        scenario_id: impl Into<String>,
        channel_names: Vec<String>,
        dt: f64,
        envelope_spec: EnvelopeSpec,
        settings: EngineSettings,
    ) -> Result<Self> {
        Self::new(
            scenario_id,
            channel_names,
            dt,
            envelope_spec,
            settings,
            HeuristicBankRegistry::builtin(),
        )
    }

    /// Creates a bounded online engine with an explicit heuristic bank.
    pub fn new(
        scenario_id: impl Into<String>,
        channel_names: Vec<String>,
        dt: f64,
        envelope_spec: EnvelopeSpec,
        settings: EngineSettings,
        bank_registry: HeuristicBankRegistry,
    ) -> Result<Self> {
        if channel_names.is_empty() {
            return Err(anyhow!("online engine requires at least one channel name"));
        }
        if !dt.is_finite() || dt <= 0.0 {
            return Err(anyhow!("online engine requires a positive finite dt"));
        }
        envelope_spec.validate()?;
        let residual_history = RingBuffer::new(settings.online.history_buffer_capacity)?;
        let offline_history = if settings.online.offline_history_enabled {
            Some(Vec::new())
        } else {
            None
        };
        let retrieval_index = build_retrieval_index(&bank_registry, &settings.retrieval_index);
        Ok(Self {
            scenario_id: scenario_id.into(),
            channel_names,
            dt,
            envelope_spec,
            retrieval_index,
            settings,
            bank_registry,
            residual_history,
            offline_history,
            next_step: 0,
        })
    }

    /// Returns the fixed online-history capacity.
    #[must_use]
    pub fn history_capacity(&self) -> usize {
        self.residual_history.capacity()
    }

    /// Returns the current bounded online-history length.
    #[must_use]
    pub fn online_history_len(&self) -> usize {
        self.residual_history.len()
    }

    /// Returns the optional offline accumulation length.
    #[must_use]
    pub fn offline_history_len(&self) -> Option<usize> {
        self.offline_history.as_ref().map(Vec::len)
    }

    /// Returns the optional full offline residual history.
    #[must_use]
    pub fn offline_residual_history(&self) -> Option<&[ResidualSample]> {
        self.offline_history.as_deref()
    }

    /// Pushes one residual sample into the bounded live engine and returns the current status.
    pub fn push_residual_sample(&mut self, time: f64, values: &[Real]) -> Result<LiveEngineStatus> {
        if values.len() != self.channel_names.len() {
            return Err(anyhow!(
                "online engine expected {} channels but received {}",
                self.channel_names.len(),
                values.len()
            ));
        }
        let values_f64 = values
            .iter()
            .map(|value| f64::from(*value))
            .collect::<Vec<_>>();
        let sample = ResidualSample {
            step: self.next_step,
            time,
            norm: euclidean_norm(&values_f64),
            values: values_f64,
        };
        self.next_step += 1;
        self.residual_history.push(sample.clone());
        if let Some(history) = &mut self.offline_history {
            history.push(sample);
        }

        let residual = ResidualTrajectory {
            scenario_id: self.scenario_id.clone(),
            channel_names: self.channel_names.clone(),
            samples: self.residual_history.iter().cloned().collect(),
        };
        let derivative_residual = smooth_residual_trajectory(&residual, &self.settings.smoothing);
        let drift = compute_drift_trajectory(&derivative_residual, self.dt, &self.scenario_id);
        let slew = compute_slew_trajectory(&derivative_residual, self.dt, &self.scenario_id);
        let sign = construct_signs(&residual, &drift, &slew);
        let envelope = build_envelope(&residual, &self.envelope_spec, &self.scenario_id);
        let grammar = evaluate_grammar_layer(&residual, &envelope);
        let syntax = characterize_syntax_with_coordination_configured(
            &sign,
            &grammar,
            None,
            &self.settings.syntax,
        );
        let semantics = retrieve_semantics_with_context(SemanticRetrievalContext {
            scenario_id: &self.scenario_id,
            syntax: &syntax,
            grammar: &grammar,
            coordinated: None,
            registry: &self.bank_registry,
            settings: &self.settings.semantics,
            index_settings: &self.settings.retrieval_index,
            index: Some(&self.retrieval_index),
        });
        self.status_from_latest(&sign, &grammar, &syntax, &semantics)
    }

    fn status_from_latest(
        &self,
        sign: &crate::engine::types::SignTrajectory,
        grammar: &[GrammarStatus],
        syntax: &SyntaxCharacterization,
        semantics: &SemanticMatchResult,
    ) -> Result<LiveEngineStatus> {
        let latest_sign = sign
            .samples
            .last()
            .cloned()
            .context("online engine did not produce a sign sample")?;
        let latest_grammar = grammar
            .last()
            .cloned()
            .context("online engine did not produce a grammar status")?;
        Ok(LiveEngineStatus {
            schema_version: LIVE_ENGINE_STATUS_SCHEMA_VERSION.to_string(),
            scenario_id: self.scenario_id.clone(),
            step: latest_sign.step,
            time: latest_sign.time,
            history_buffer_capacity: self.history_capacity(),
            current_history_len: self.online_history_len(),
            offline_history_len: self.offline_history_len(),
            numeric_mode: self.settings.online.numeric_mode.clone(),
            residual_norm: latest_sign.residual_norm,
            drift_norm: latest_sign.drift_norm,
            slew_norm: latest_sign.slew_norm,
            projection: latest_sign.projection,
            syntax_label: syntax.trajectory_label.clone(),
            grammar_state: latest_grammar.state,
            grammar_reason_code: latest_grammar.reason_code,
            grammar_reason_text: latest_grammar.reason_text,
            trust_scalar: latest_grammar.trust_scalar.value(),
            semantic_disposition_code: match semantics.disposition {
                SemanticDisposition::Match => 0,
                SemanticDisposition::CompatibleSet => 1,
                SemanticDisposition::Ambiguous => 2,
                SemanticDisposition::Unknown => 3,
            },
            semantic_disposition: format!("{:?}", semantics.disposition),
            selected_heuristic_ids: semantics.selected_heuristic_ids.clone(),
            note: "Status derives from the bounded online window only. Optional offline accumulation remains separate from the memory-bounded live path.".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{numeric_mode_label, OnlineStructuralEngine, Real, RingBuffer};
    use crate::engine::settings::EngineSettings;
    use crate::engine::types::EnvelopeMode;
    use crate::math::envelope::EnvelopeSpec;

    #[test]
    fn ring_buffer_capacity_fixed() {
        let buffer = RingBuffer::<i32>::new(3).expect("buffer");
        assert_eq!(buffer.capacity(), 3);
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn ring_buffer_overwrites_oldest_entries() {
        let mut buffer = RingBuffer::new(2).expect("buffer");
        assert_eq!(buffer.push(1), None);
        assert_eq!(buffer.push(2), None);
        assert_eq!(buffer.push(3), Some(1));
        assert_eq!(buffer.iter().copied().collect::<Vec<_>>(), vec![2, 3]);
    }

    #[test]
    fn online_engine_memory_history_bounded() {
        let mut settings = EngineSettings::default();
        settings.online.history_buffer_capacity = 4;
        let mut engine = OnlineStructuralEngine::with_builtin_bank(
            "bounded",
            vec!["x".to_string()],
            1.0,
            EnvelopeSpec {
                name: "fixed".to_string(),
                mode: EnvelopeMode::Fixed,
                base_radius: 1.0,
                slope: 0.0,
                switch_step: None,
                secondary_slope: None,
                secondary_base: None,
            },
            settings,
        )
        .expect("engine");
        for step in 0..16 {
            engine
                .push_residual_sample(step as f64, &[step as Real * 0.01])
                .expect("status");
        }
        assert_eq!(engine.online_history_len(), 4);
        assert_eq!(engine.history_capacity(), 4);
    }

    #[test]
    fn syntax_computation_works_with_bounded_history() {
        let mut settings = EngineSettings::default();
        settings.online.history_buffer_capacity = 6;
        let mut engine = OnlineStructuralEngine::with_builtin_bank(
            "syntax",
            vec!["x".to_string()],
            1.0,
            EnvelopeSpec {
                name: "fixed".to_string(),
                mode: EnvelopeMode::Fixed,
                base_radius: 0.8,
                slope: 0.0,
                switch_step: None,
                secondary_slope: None,
                secondary_base: None,
            },
            settings,
        )
        .expect("engine");
        let mut last_label = String::new();
        for step in 0..12 {
            let status = engine
                .push_residual_sample(step as f64, &[0.1 + step as Real * 0.02])
                .expect("status");
            last_label = status.syntax_label;
        }
        assert!(!last_label.is_empty());
    }

    #[test]
    fn offline_export_path_is_separate_from_online_buffer_if_applicable() {
        let mut settings = EngineSettings::default();
        settings.online.history_buffer_capacity = 3;
        settings.online.offline_history_enabled = true;
        let mut engine = OnlineStructuralEngine::with_builtin_bank(
            "offline",
            vec!["x".to_string()],
            1.0,
            EnvelopeSpec {
                name: "fixed".to_string(),
                mode: EnvelopeMode::Fixed,
                base_radius: 1.0,
                slope: 0.0,
                switch_step: None,
                secondary_slope: None,
                secondary_base: None,
            },
            settings,
        )
        .expect("engine");
        for step in 0..8 {
            engine
                .push_residual_sample(step as f64, &[0.2 + step as Real * 0.01])
                .expect("status");
        }
        assert_eq!(engine.online_history_len(), 3);
        assert_eq!(engine.offline_history_len(), Some(8));
        assert_eq!(
            engine
                .offline_residual_history()
                .expect("offline history")
                .len(),
            8
        );
    }

    #[test]
    fn numeric_mode_reports_compile_time_selection() {
        assert!(matches!(numeric_mode_label(), "f32" | "f64"));
    }
}
