//! Dedicated deterministic CSV replay driver for the dashboard surface.
//!
//! This driver routes observed/predicted CSV inputs through the same deterministic engine path,
//! then exposes explicit replay controls without relying on wall-clock timing.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::cli::args::CsvInputConfig;
use crate::engine::bank::HeuristicBankRegistry;
use crate::engine::config::{BankSourceConfig, CommonRunConfig};
use crate::engine::pipeline::{EngineConfig, StructuralSemioticsEngine};
use crate::engine::residual_layer::extract_residuals;
use crate::engine::settings::EngineSettings;
use crate::engine::types::{EngineOutputBundle, GrammarState};
use crate::io::input::load_csv_trajectories;
use crate::live::{to_real, OnlineStructuralEngine};

use super::build::{grammar_state_label, joined_candidates, joined_selected_heuristics};
use super::render::{render_frame_ascii, ReplayFrameChrome};
use super::types::{
    DashboardReplayConfig, DashboardReplayEvent, DashboardReplayStream,
    DASHBOARD_EVENT_SCHEMA_VERSION,
};

/// Stable schema identifier for deterministic CSV replay state snapshots.
pub const CSV_REPLAY_STATE_SCHEMA_VERSION: &str = "dsfb-semiotics-csv-replay-state/v1";

/// Current replay run-state.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CsvReplayRunState {
    Running,
    Paused,
    Ended,
}

impl CsvReplayRunState {
    fn as_label(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Ended => "ended",
        }
    }
}

/// Deterministic timing state exported by the CSV replay driver.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CsvReplayTimingState {
    pub schema_version: String,
    pub scenario_id: String,
    pub source_label: String,
    pub current_frame_index: usize,
    pub total_frames: usize,
    pub current_time: f64,
    pub playback_speed: f64,
    pub run_state: CsvReplayRunState,
    pub paused: bool,
    pub end_of_stream: bool,
}

/// Deterministic CSV replay driver with explicit control state.
#[derive(Clone, Debug)]
pub struct CsvReplayDriver {
    config: DashboardReplayConfig,
    stream: DashboardReplayStream,
    frame_index: usize,
    run_state: CsvReplayRunState,
    accumulator_seconds: f64,
    emitted_markers: Vec<String>,
}

impl CsvReplayDriver {
    /// Builds a CSV replay driver directly from the observed/predicted CSV path configuration.
    pub fn from_csv_run(
        common: CommonRunConfig,
        input: CsvInputConfig,
        settings: EngineSettings,
        config: DashboardReplayConfig,
    ) -> Result<Self> {
        config.validate()?;
        input.validate()?;

        let bundle = StructuralSemioticsEngine::with_settings(
            EngineConfig::csv(common.clone(), input.clone()),
            settings.clone(),
        )?
        .run_selected()?;
        Self::from_bundle_and_csv_input(&bundle, &common, &input, &settings, config)
    }

    /// Builds a CSV replay driver from an already-executed CSV bundle plus the originating CSV
    /// input configuration.
    pub fn from_bundle_and_csv_input(
        bundle: &EngineOutputBundle,
        common: &CommonRunConfig,
        input: &CsvInputConfig,
        settings: &EngineSettings,
        config: DashboardReplayConfig,
    ) -> Result<Self> {
        config.validate()?;
        if bundle.run_metadata.input_mode != "csv" {
            return Err(anyhow!(
                "CSV replay driver requires a bundle produced from CSV ingestion"
            ));
        }
        input.validate()?;

        let scenario = bundle
            .scenario_outputs
            .iter()
            .find(|scenario| {
                config
                    .scenario_filter
                    .as_ref()
                    .map(|filter| &scenario.record.id == filter)
                    .unwrap_or(true)
            })
            .ok_or_else(|| anyhow!("CSV replay scenario filter did not match any scenario"))?;
        let baselines = bundle
            .evaluation
            .baseline_results
            .iter()
            .filter(|result| result.scenario_id == scenario.record.id)
            .cloned()
            .collect::<Vec<_>>();

        let (observed, predicted) = load_csv_trajectories(input)?;
        let residual = extract_residuals(&observed, &predicted, &scenario.record.id);
        let bank_registry = load_bank_registry(common)?;
        let mut engine = OnlineStructuralEngine::new(
            scenario.record.id.clone(),
            residual.channel_names.clone(),
            common.dt,
            input.envelope_spec()?,
            settings.clone(),
            bank_registry,
        )?;

        let mut events = Vec::with_capacity(residual.samples.len());
        let mut recent_log = Vec::new();
        let mut previous_syntax = None::<String>;
        let mut previous_grammar_state = None::<GrammarState>;
        let mut previous_reason = None::<String>;
        let mut previous_semantic = None::<String>;
        let mut trust_was_below = false;
        let total_frames = residual.samples.len();

        for sample in &residual.samples {
            let real_values = sample
                .values
                .iter()
                .map(|value| to_real(*value))
                .collect::<Vec<_>>();
            let status = engine.push_residual_sample(sample.time, &real_values)?;

            let mut markers = Vec::new();
            if sample.step == 0 {
                let marker = format!("start scenario={} t={:.3}", scenario.record.id, sample.time);
                markers.push(marker.clone());
                recent_log.push(marker);
            }
            if previous_syntax.as_ref() != Some(&status.syntax_label) {
                let marker = format!("syntax -> {} at step {}", status.syntax_label, status.step);
                markers.push(marker.clone());
                recent_log.push(marker);
                previous_syntax = Some(status.syntax_label.clone());
            }
            if previous_grammar_state != Some(status.grammar_state) {
                let marker = format!(
                    "grammar -> {} at step {}",
                    grammar_state_label(status.grammar_state),
                    status.step
                );
                markers.push(marker.clone());
                recent_log.push(marker);
                previous_grammar_state = Some(status.grammar_state);
            }
            if previous_reason.as_ref() != Some(&status.grammar_reason_text) {
                let marker = format!(
                    "grammar_reason -> {} at step {}",
                    status.grammar_reason_text, status.step
                );
                markers.push(marker.clone());
                recent_log.push(marker);
                previous_reason = Some(status.grammar_reason_text.clone());
            }
            if previous_semantic.as_ref() != Some(&status.semantic_disposition) {
                let marker = format!(
                    "semantic -> {} at step {}",
                    status.semantic_disposition, status.step
                );
                markers.push(marker.clone());
                recent_log.push(marker);
                previous_semantic = Some(status.semantic_disposition.clone());
            }
            for comparator in &baselines {
                if comparator.triggered && comparator.first_trigger_step == Some(sample.step) {
                    let marker = format!(
                        "{} alarm at t={:.3}",
                        comparator.comparator_label,
                        comparator.first_trigger_time.unwrap_or(sample.time)
                    );
                    markers.push(marker.clone());
                    recent_log.push(marker);
                }
            }
            let trust_is_below = status.trust_scalar <= config.trust_threshold;
            if trust_is_below && !trust_was_below {
                let marker = format!(
                    "trust threshold {:.2} crossed at t={:.3}",
                    config.trust_threshold, sample.time
                );
                markers.push(marker.clone());
                recent_log.push(marker);
            }
            trust_was_below = trust_is_below;

            let comparator_alarms = baselines
                .iter()
                .filter(|result| {
                    result.triggered
                        && result
                            .first_trigger_step
                            .map(|step| step <= sample.step)
                            .unwrap_or(false)
                })
                .map(|result| result.comparator_label.clone())
                .collect::<Vec<_>>();
            let recent = recent_log
                .iter()
                .rev()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>();

            events.push(DashboardReplayEvent {
                schema_version: DASHBOARD_EVENT_SCHEMA_VERSION.to_string(),
                engine_version: bundle.run_metadata.crate_version.clone(),
                bank_version: bundle.run_metadata.bank.bank_version.clone(),
                scenario_id: scenario.record.id.clone(),
                scenario_title: scenario.record.title.clone(),
                frame_index: sample.step,
                total_frames,
                step: status.step,
                time: status.time,
                residual_norm: status.residual_norm,
                drift_norm: status.drift_norm,
                slew_norm: status.slew_norm,
                projection_1: status.projection[0],
                projection_2: status.projection[1],
                projection_3: status.projection[2],
                syntax_label: status.syntax_label.clone(),
                grammar_state: grammar_state_label(status.grammar_state).to_string(),
                grammar_margin: scenario
                    .grammar
                    .get(sample.step)
                    .map(|grammar| grammar.margin)
                    .unwrap_or(0.0),
                grammar_reason_text: status.grammar_reason_text.clone(),
                trust_scalar: status.trust_scalar,
                semantic_disposition: status.semantic_disposition.clone(),
                semantic_candidates: joined_candidates(scenario),
                selected_heuristics: if status.selected_heuristic_ids.is_empty() {
                    joined_selected_heuristics(scenario)
                } else {
                    status.selected_heuristic_ids.join(" | ")
                },
                admissibility_audit: scenario.semantics.retrieval_audit.note.clone(),
                comparator_alarms: if comparator_alarms.is_empty() {
                    "none".to_string()
                } else {
                    comparator_alarms.join(" | ")
                },
                event_markers: markers,
                event_log: if recent.is_empty() {
                    "no transitions yet".to_string()
                } else {
                    recent.join(" | ")
                },
            });
        }

        if let Some(max_frames) = config.max_frames {
            events.truncate(max_frames.min(events.len()));
        }
        if events.is_empty() {
            return Err(anyhow!(
                "CSV replay driver did not produce any replay events"
            ));
        }

        Ok(Self {
            frame_index: 0,
            run_state: if config.start_paused {
                CsvReplayRunState::Paused
            } else {
                CsvReplayRunState::Running
            },
            accumulator_seconds: 0.0,
            emitted_markers: events[0].event_markers.clone(),
            stream: DashboardReplayStream {
                schema_version: DASHBOARD_EVENT_SCHEMA_VERSION.to_string(),
                engine_version: bundle.run_metadata.crate_version.clone(),
                bank_version: bundle.run_metadata.bank.bank_version.clone(),
                input_mode: "csv".to_string(),
                scenario_id: scenario.record.id.clone(),
                scenario_title: scenario.record.title.clone(),
                source_label: config.source_label.clone().unwrap_or_else(|| {
                    format!(
                        "{} | {}",
                        input.observed_csv.display(),
                        input.predicted_csv.display()
                    )
                }),
                events,
            },
            config,
        })
    }

    /// Returns the current replay stream.
    #[must_use]
    pub fn stream(&self) -> &DashboardReplayStream {
        &self.stream
    }

    /// Returns the current replay event.
    #[must_use]
    pub fn current_event(&self) -> &DashboardReplayEvent {
        &self.stream.events[self.frame_index]
    }

    /// Returns the emitted event markers accumulated so far.
    #[must_use]
    pub fn emitted_markers(&self) -> &[String] {
        &self.emitted_markers
    }

    /// Returns the current replay timing state.
    #[must_use]
    pub fn timing_state(&self) -> CsvReplayTimingState {
        let current = self.current_event();
        CsvReplayTimingState {
            schema_version: CSV_REPLAY_STATE_SCHEMA_VERSION.to_string(),
            scenario_id: self.stream.scenario_id.clone(),
            source_label: self.stream.source_label.clone(),
            current_frame_index: self.frame_index,
            total_frames: self.stream.events.len(),
            current_time: current.time,
            playback_speed: self.config.playback_speed,
            run_state: self.run_state,
            paused: matches!(self.run_state, CsvReplayRunState::Paused),
            end_of_stream: matches!(self.run_state, CsvReplayRunState::Ended),
        }
    }

    /// Returns whether replay is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        matches!(self.run_state, CsvReplayRunState::Paused)
    }

    /// Returns whether replay reached the end of the stream.
    #[must_use]
    pub fn is_ended(&self) -> bool {
        matches!(self.run_state, CsvReplayRunState::Ended)
    }

    /// Pauses replay.
    pub fn pause(&mut self) {
        if !self.is_ended() {
            self.run_state = CsvReplayRunState::Paused;
        }
    }

    /// Resumes replay.
    pub fn resume(&mut self) {
        if !self.is_ended() {
            self.run_state = CsvReplayRunState::Running;
        }
    }

    /// Sets the deterministic playback speed.
    pub fn set_playback_speed(&mut self, playback_speed: f64) -> Result<()> {
        if !playback_speed.is_finite() || playback_speed <= 0.0 {
            return Err(anyhow!(
                "CSV replay playback speed must be positive and finite; got {}",
                playback_speed
            ));
        }
        self.config.playback_speed = playback_speed;
        Ok(())
    }

    /// Advances one event forward.
    pub fn single_step(&mut self) -> bool {
        let next_state = if self.is_paused() {
            CsvReplayRunState::Paused
        } else {
            CsvReplayRunState::Running
        };
        self.advance_index_by(1, next_state)
    }

    /// Steps one event backward when a previous frame exists.
    pub fn step_backward(&mut self) -> bool {
        if self.frame_index == 0 {
            return false;
        }
        self.frame_index -= 1;
        self.run_state = CsvReplayRunState::Paused;
        true
    }

    /// Advances the deterministic replay clock by the requested virtual seconds.
    pub fn advance(&mut self, delta_seconds: f64) -> Result<usize> {
        if !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return Err(anyhow!(
                "CSV replay delta_seconds must be finite and non-negative; got {}",
                delta_seconds
            ));
        }
        if self.is_paused() || self.is_ended() {
            return Ok(0);
        }
        self.accumulator_seconds += delta_seconds * self.config.playback_speed;
        let mut advanced = 0usize;
        loop {
            let Some(duration) = self.next_frame_duration() else {
                self.run_state = CsvReplayRunState::Ended;
                break;
            };
            if self.accumulator_seconds + 1.0e-12 < duration {
                break;
            }
            self.accumulator_seconds -= duration;
            if !self.advance_index_by(1, CsvReplayRunState::Running) {
                break;
            }
            advanced += 1;
            if self.is_ended() {
                break;
            }
        }
        Ok(advanced)
    }

    /// Renders the current frame as deterministic ASCII.
    #[must_use]
    pub fn render_current_frame_ascii(&self) -> String {
        render_frame_ascii(
            &self.stream,
            self.current_event(),
            self.config.width.max(80),
            self.config.height.max(24),
            &ReplayFrameChrome {
                replay_mode_label: "REPLAY MODE: CSV",
                source_label: &self.stream.source_label,
                status_label: self.run_state.as_label(),
                playback_speed: self.config.playback_speed,
                current_time: self.current_event().time,
            },
        )
    }

    /// Renders the replay as a deterministic ASCII walkthrough under the current driver state.
    pub fn render_replay_ascii(&self) -> String {
        let mut clone = self.clone();
        let frame_limit = clone
            .config
            .max_frames
            .unwrap_or(clone.stream.events.len())
            .min(clone.stream.events.len());
        let mut frames = vec![clone.render_current_frame_ascii()];
        while frames.len() < frame_limit && !clone.is_ended() && !clone.is_paused() {
            if !clone.single_step() {
                break;
            }
            frames.push(clone.render_current_frame_ascii());
        }
        frames.join("\n\n")
    }

    fn next_frame_duration(&self) -> Option<f64> {
        if self.frame_index + 1 >= self.stream.events.len() {
            return None;
        }
        let current = &self.stream.events[self.frame_index];
        let next = &self.stream.events[self.frame_index + 1];
        Some((next.time - current.time).max(1.0e-9))
    }

    fn advance_index_by(&mut self, delta: usize, next_state: CsvReplayRunState) -> bool {
        if self.frame_index + delta >= self.stream.events.len() {
            self.frame_index = self.stream.events.len().saturating_sub(1);
            self.run_state = CsvReplayRunState::Ended;
            return false;
        }
        self.frame_index += delta;
        self.run_state = if self.frame_index + 1 >= self.stream.events.len() {
            CsvReplayRunState::Ended
        } else {
            next_state
        };
        if matches!(self.run_state, CsvReplayRunState::Ended) {
            self.run_state = CsvReplayRunState::Ended;
        }
        let markers = self.current_event().event_markers.clone();
        self.emitted_markers.extend(markers);
        true
    }
}

fn load_bank_registry(common: &CommonRunConfig) -> Result<HeuristicBankRegistry> {
    match &common.bank.source {
        BankSourceConfig::Builtin => {
            Ok(HeuristicBankRegistry::load_builtin(common.bank.is_strict())?.0)
        }
        BankSourceConfig::External(path) => {
            Ok(HeuristicBankRegistry::load_external_json(path, common.bank.is_strict())?.0)
        }
    }
}
