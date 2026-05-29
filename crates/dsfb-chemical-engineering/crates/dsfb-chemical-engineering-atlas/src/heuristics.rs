//! Chemical process-residual heuristic records — H1–H6.
//!
//! Each [`ChemicalProcessHeuristicRecordV1`] is the *record* form of a heuristic: the residual motif,
//! the explicit drift / slew / admissibility conditions (as inspectable strings), the operator-facing
//! candidate label, and the documented false-positive and false-negative modes. The **executable**
//! predicates that test these conditions against a fused episode live in the `edge` crate
//! (`heuristics::evaluate_rules`); keeping only the string schema here means this authority crate has
//! no dependency on any execution type (`FusedEpisode`, `GrammarState`). The edge subset gate proves
//! edge's runtime rules mirror these records.
//!
//! **Every label is a candidate hypothesis for operator review, never a proven root cause.**

/// One curated process-heuristic record (schema v1). `&'static` throughout so the set is `const`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ChemicalProcessHeuristicRecordV1 {
    pub heuristic_id: &'static str,
    pub name: &'static str,
    pub applicable_process_type: &'static [&'static str],
    pub required_variables: &'static [&'static str],
    pub detector_inputs: &'static [&'static str],
    pub residual_pattern: &'static str,
    pub drift_condition: &'static str,
    pub slew_condition: &'static str,
    pub admissibility_condition: &'static str,
    pub episode_label: &'static str,
    pub operator_explanation: &'static str,
    pub known_false_positive_modes: &'static [&'static str],
    pub known_false_negative_modes: &'static [&'static str],
    pub severity_policy: &'static str,
    pub engineering_basis: &'static str,
}

/// H1–H6, in priority order (H1 first). Mirrors `edge::heuristics::canonical_bank()` metadata; the
/// `drift_/slew_/admissibility_condition` strings make `edge::heuristics::evaluate_rules` inspectable.
pub const HEURISTIC_RECORDS: &[ChemicalProcessHeuristicRecordV1; 6] = &[
    ChemicalProcessHeuristicRecordV1 {
        heuristic_id: "H1",
        name: "Sensor bias / calibration drift candidate",
        applicable_process_type: &["continuous", "batch", "spectroscopic"],
        required_variables: &["any monitored channel with structurally-related neighbours"],
        detector_inputs: &["sensor_bias", "shewhart_max_z", "robust_z_mad"],
        residual_pattern: "one variable drifts; correlated variables do not follow; PCA contribution dominated by one variable; balance residuals stable",
        drift_condition: "single-variable isolation (sensor_bias fires)",
        slew_condition: "not required",
        admissibility_condition: "co_drift absent; balance residuals stable",
        episode_label: "likely sensor bias / calibration drift",
        operator_explanation: "A single channel is deviating while structurally-related channels stay nominal -- consistent with instrument bias rather than a process upset.",
        known_false_positive_modes: &["a genuinely local process change affecting one measurement point", "intermittent comms dropouts"],
        known_false_negative_modes: &["simultaneous multi-sensor calibration drift presents as co-drift, not isolation"],
        severity_policy: "advisory; recommend cross-check against redundant instrument",
        engineering_basis: "single-variable contribution isolation in MSPC contribution analysis",
    },
    ChemicalProcessHeuristicRecordV1 {
        heuristic_id: "H2",
        name: "Actuator stiction / valve stick-slip candidate",
        applicable_process_type: &["control loops", "actuated valves"],
        required_variables: &["valve position", "controller output"],
        detector_inputs: &["page_hinkley_spe", "cusum_spe", "spectral_entropy_spe"],
        residual_pattern: "residual slew snaps after a delay; oscillatory correction follows; spectral content shifts",
        drift_condition: "not required",
        slew_condition: "peak_slew > 1.0 (abrupt residual snap)",
        admissibility_condition: "page_hinkley_spe and spectral_entropy_spe both fire",
        episode_label: "candidate actuator stiction / valve stick-slip",
        operator_explanation: "An abrupt residual slew followed by oscillatory recovery and a spectral-entropy shift is consistent with valve stiction releasing under controller pressure.",
        known_false_positive_modes: &["legitimate setpoint changes", "recipe transitions", "aggressive controller tuning"],
        known_false_negative_modes: &["slow stiction without limit-cycle oscillation may not produce a slew snap"],
        severity_policy: "advisory; correlate with valve position vs controller output",
        engineering_basis: "stick-slip limit cycles in DAMADICS-class actuator FDI literature",
    },
    ChemicalProcessHeuristicRecordV1 {
        heuristic_id: "H3",
        name: "Reactor thermal-excursion candidate",
        applicable_process_type: &["reactors", "exothermic processes"],
        required_variables: &["reactor temperature", "cooling/heating variables"],
        detector_inputs: &["co_drift", "ewma_spe", "pca_t2"],
        residual_pattern: "temperature-group residual drift with co-drift of cooling/heating variables and rising slew",
        drift_condition: "dominant motif = DriftAccum; ewma_spe fires",
        slew_condition: "rising slew (accelerating)",
        admissibility_condition: "co_drift fires across the thermal variable group",
        episode_label: "candidate reactor thermal excursion",
        operator_explanation: "Co-drifting thermal variables with accelerating slew are consistent with an emerging thermal excursion; cross-check conversion/quality proxies.",
        known_false_positive_modes: &["ambient/utility transients", "planned ramp", "feed preheat changes"],
        known_false_negative_modes: &["fast runaway can outpace the drift window", "very slow excursions look nominal"],
        severity_policy: "advisory; this carries NO trip authority and must not gate safety interlocks",
        engineering_basis: "co-monotone thermal residual signatures in CSTR/TEP-class benchmarks",
    },
    ChemicalProcessHeuristicRecordV1 {
        heuristic_id: "H4",
        name: "Feed-composition disturbance candidate",
        applicable_process_type: &["continuous plantwide"],
        required_variables: &["feed composition/flow", "downstream quality proxies"],
        detector_inputs: &["co_drift", "pca_spe_q", "mann_kendall_spe"],
        residual_pattern: "upstream composition/flow residual breach; downstream contribution migration; detector agreement grows over propagation time",
        drift_condition: "mann_kendall_spe trend, recruiting >= 3 detector families over propagation time",
        slew_condition: "not required",
        admissibility_condition: "co_drift fires with rising pca_spe_q residual energy",
        episode_label: "candidate feed-composition disturbance (propagating)",
        operator_explanation: "A residual that emerges upstream and recruits downstream detectors over time is consistent with a propagating feed-composition disturbance.",
        known_false_positive_modes: &["coordinated multi-unit setpoint moves", "shared-utility disturbances"],
        known_false_negative_modes: &["fast-mixing processes may not show a clear propagation delay"],
        severity_policy: "advisory; inspect upstream analyser and feed lineup",
        engineering_basis: "fault-propagation contribution migration in plantwide MSPC",
    },
    ChemicalProcessHeuristicRecordV1 {
        heuristic_id: "H5",
        name: "Batch-phase misalignment candidate",
        applicable_process_type: &["batch", "fed-batch", "fermentation"],
        required_variables: &["phase / recipe-stage label"],
        detector_inputs: &["pca_spe_q", "psi_spe", "ks_spe"],
        residual_pattern: "large residuals only at phase boundaries; phase-aligned envelope reduces anomaly",
        drift_condition: "not required",
        slew_condition: "not required",
        admissibility_condition: "psi_spe, ks_spe and pca_spe_q all fire (distribution shift localised to phase boundary)",
        episode_label: "candidate batch-phase misalignment",
        operator_explanation: "Residuals concentrated at phase transitions, distributionally distinct from steady phases, are consistent with batch alignment/recipe-stage timing rather than a true fault.",
        known_false_positive_modes: &["genuine phase-localised faults", "recipe edits"],
        known_false_negative_modes: &["misalignment uniformly across a phase may not localise to boundaries"],
        severity_policy: "advisory; recommend phase-aligned re-evaluation before escalation",
        engineering_basis: "multiway/phase-aligned PCA in batch process monitoring (e.g. IndPenSim)",
    },
    ChemicalProcessHeuristicRecordV1 {
        heuristic_id: "H6",
        name: "Controller-compensation masking candidate",
        applicable_process_type: &["closed-loop controlled processes"],
        required_variables: &["controlled variable", "manipulated variable"],
        detector_inputs: &["pca_t2", "ewma_spe", "co_drift"],
        residual_pattern: "controlled variable appears stable; manipulated-variable residual grows; latent residual changes before any raw limit breach",
        drift_condition: "ewma_spe latent trend while the controlled variable stays in-band",
        slew_condition: "not required",
        admissibility_condition: "dominant motif = BoundaryGrazing; pca_t2 and co_drift fire",
        episode_label: "candidate controller-compensation masking",
        operator_explanation: "Latent (score-space) residual growth while the controlled variable stays in-band can indicate the control system is absorbing growing process stress -- visible structurally before a raw alarm.",
        known_false_positive_modes: &["normal load-following", "disturbance rejection within design envelope"],
        known_false_negative_modes: &["open-loop or saturated actuators remove the masking signature"],
        severity_policy: "advisory; surface manipulated-variable headroom to operators",
        engineering_basis: "score-space vs raw-space residual divergence under closed-loop control",
    },
];
