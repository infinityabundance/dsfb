use crate::engine::types::{
    DetectabilityBoundInputs, EnvelopeMode, GroupDefinition, ScenarioRecord,
};
use crate::math::envelope::EnvelopeSpec;

#[derive(Clone, Copy, Debug)]
pub enum ClaimClass {
    TheoremAligned,
    IllustrativeExample,
    SyntheticExperiment,
}

impl ClaimClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TheoremAligned => "theorem-aligned demonstration",
            Self::IllustrativeExample => "illustrative example",
            Self::SyntheticExperiment => "synthetic experiment",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ScenarioKind {
    NominalStable,
    GradualDegradation,
    CurvatureOnset,
    AbruptEvent,
    OscillatoryBounded,
    OutwardExitA,
    OutwardExitB,
    OutwardExitC,
    InwardInvariance,
    GroupedCorrelated,
    RegimeSwitch,
    NoisyStructured,
    MagnitudeMatchedAdmissible,
    MagnitudeMatchedDetectable,
}

#[derive(Clone, Debug)]
pub struct ScenarioDefinition {
    pub kind: ScenarioKind,
    pub record: ScenarioRecord,
    pub channels: Vec<String>,
    pub envelope_spec: EnvelopeSpec,
    pub detectability_inputs: Option<DetectabilityBoundInputs>,
    pub groups: Vec<GroupDefinition>,
    pub aggregate_envelope_spec: Option<EnvelopeSpec>,
}

pub fn all_scenarios() -> Vec<ScenarioDefinition> {
    vec![
        scenario_nominal_stable(),
        scenario_gradual_degradation(),
        scenario_curvature_onset(),
        scenario_abrupt_event(),
        scenario_oscillatory_bounded(),
        scenario_outward_exit_a(),
        scenario_outward_exit_b(),
        scenario_outward_exit_c(),
        scenario_inward_invariance(),
        scenario_grouped_correlated(),
        scenario_regime_switch(),
        scenario_noisy_structured(),
        scenario_magnitude_matched_admissible(),
        scenario_magnitude_matched_detectable(),
    ]
}

fn scenario_nominal_stable() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::NominalStable,
        record: record(
            "nominal_stable",
            "Nominal Stable Case",
            "Reference admissible trajectory for comparison, deterministic repeatability, and layer-audit baselining.",
            "Residuals stay inside a fixed admissibility envelope, illustrating grammatical nominal behavior rather than anomaly detection.",
            ClaimClass::SyntheticExperiment,
            "This is a constructed nominal baseline, not evidence of field performance.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "nominal_fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 0.34,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_gradual_degradation() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::GradualDegradation,
        record: record(
            "gradual_degradation",
            "Gradual Degradation / Monotone Drift",
            "Illustrate monotone drift as a semiotic syntax motif and semantics-bank retrieval candidate.",
            "Drift-dominated residual evolution illustrates the syntax layer without claiming unique diagnosis.",
            ClaimClass::IllustrativeExample,
            "The slow-drift interpretation remains a conservative heuristic candidate, not a proof of degradation cause.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "gradual_widening".to_string(),
            mode: EnvelopeMode::Widening,
            base_radius: 0.25,
            slope: 0.0013,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_curvature_onset() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::CurvatureOnset,
        record: record(
            "curvature_onset",
            "Curvature Onset Case",
            "Show how similar low-order residual magnitudes can acquire different syntax once slew grows materially.",
            "Curvature-dominated residual evolution illustrates the drift/slew syntax distinction.",
            ClaimClass::IllustrativeExample,
            "The scenario is synthetic and only demonstrates a curvature-rich motif under controlled sampling.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "curvature_fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 0.52,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_abrupt_event() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::AbruptEvent,
        record: record(
            "abrupt_event",
            "Abrupt Event / Slew Spike",
            "Demonstrate a localized high-slew motif and its conservative semantics-bank interpretation.",
            "Localized slew spikes create a syntax signature distinct from gradual drift.",
            ClaimClass::IllustrativeExample,
            "A localized slew spike may be compatible with several causes; the semantics layer keeps that ambiguity explicit.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "abrupt_fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 0.64,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_oscillatory_bounded() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::OscillatoryBounded,
        record: record(
            "oscillatory_bounded",
            "Oscillatory Bounded Case",
            "Illustrate structured but grammatical oscillation inside a fixed envelope.",
            "Detectability is not asserted here because the trajectory remains admissible by construction.",
            ClaimClass::SyntheticExperiment,
            "The example demonstrates admissible oscillation only for the configured envelope and sample rate.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "oscillatory_fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 0.42,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_outward_exit_a() -> ScenarioDefinition {
    linear_exit_case(
        "outward_exit_case_a",
        "Envelope Exit Under Sustained Outward Drift",
        0.42,
        0.0010,
        0.15,
        0.0042,
    )
}

fn scenario_outward_exit_b() -> ScenarioDefinition {
    linear_exit_case(
        "outward_exit_case_b",
        "Residual-Envelope Bound Case B",
        0.41,
        0.0008,
        0.12,
        0.0050,
    )
}

fn scenario_outward_exit_c() -> ScenarioDefinition {
    linear_exit_case(
        "outward_exit_case_c",
        "Residual-Envelope Bound Case C",
        0.39,
        0.0005,
        0.18,
        0.0038,
    )
}

fn linear_exit_case(
    id: &str,
    title: &str,
    envelope_base: f64,
    envelope_slope: f64,
    residual_base: f64,
    residual_slope: f64,
) -> ScenarioDefinition {
    ScenarioDefinition {
        kind: match id {
            "outward_exit_case_a" => ScenarioKind::OutwardExitA,
            "outward_exit_case_b" => ScenarioKind::OutwardExitB,
            _ => ScenarioKind::OutwardExitC,
        },
        record: record(
            id,
            title,
            "Quantitatively illustrate finite-time envelope exit and the residual-envelope detectability upper bound.",
            "This scenario is theorem-aligned: both residual norm and envelope radius evolve linearly so the bound can be checked directly against the sampled crossing time.",
            ClaimClass::TheoremAligned,
            "The bound is a sufficient condition under the configured monotone outward drift assumptions only.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: format!("{id}_widening"),
            mode: EnvelopeMode::Widening,
            base_radius: envelope_base,
            slope: envelope_slope,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: Some(DetectabilityBoundInputs {
            t0: 0.0,
            alpha: residual_slope,
            kappa: envelope_slope,
            delta0: envelope_base - residual_base,
        }),
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_inward_invariance() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::InwardInvariance,
        record: record(
            "inward_invariance",
            "Envelope Invariance Under Inward-Compatible Drift",
            "Demonstrate forward invariance when residual growth remains compatible with the envelope motion.",
            "This case illustrates the grammar-side invariance half of the exit-invariance pair.",
            ClaimClass::TheoremAligned,
            "It is a synthetic sufficient-condition demonstration, not a complete characterization of admissible trajectories.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "invariance_tightening".to_string(),
            mode: EnvelopeMode::Tightening,
            base_radius: 0.42,
            slope: 0.0004,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_grouped_correlated() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::GroupedCorrelated,
        record: record(
            "grouped_correlated",
            "Grouped Correlated Degradation",
            "Illustrate coordinated or common-mode structure using local and aggregate envelopes.",
            "Grouped residual structure extends the engine beyond channel-local reading into auditable coordinated semiotics.",
            ClaimClass::SyntheticExperiment,
            "The coordinated interpretation remains a group-level motif statement and does not identify the underlying shared cause uniquely.",
        ),
        channels: channels(4),
        envelope_spec: EnvelopeSpec {
            name: "group_local_widening".to_string(),
            mode: EnvelopeMode::Widening,
            base_radius: 0.34,
            slope: 0.0009,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: vec![GroupDefinition {
            group_id: "g0".to_string(),
            member_indices: vec![0, 1, 2],
        }],
        aggregate_envelope_spec: Some(EnvelopeSpec {
            name: "group_aggregate".to_string(),
            mode: EnvelopeMode::Aggregate,
            base_radius: 0.22,
            slope: 0.0005,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        }),
    }
}

fn scenario_regime_switch() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::RegimeSwitch,
        record: record(
            "regime_switch",
            "Regime-Switched Envelope Case",
            "Demonstrate how grammar status depends on envelope evolution, not residual magnitude alone.",
            "A time-varying regime-switched admissibility envelope changes grammatical interpretation while the residual remains deterministic.",
            ClaimClass::SyntheticExperiment,
            "The regime switch is scripted and illustrates auditability of envelope assumptions rather than learned adaptation.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "regime_switched".to_string(),
            mode: EnvelopeMode::RegimeSwitched,
            base_radius: 0.36,
            slope: 0.0008,
            switch_step: Some(90),
            secondary_slope: Some(0.0002),
            secondary_base: Some(0.28),
        },
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_noisy_structured() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::NoisyStructured,
        record: record(
            "noisy_structured",
            "Noisy but Structured Case",
            "Show that deterministic structured noise can coexist with an auditable layered interpretation pipeline.",
            "This case is a synthetic experiment on robustness of the deterministic artifact pipeline, not a probabilistic noise model claim.",
            ClaimClass::SyntheticExperiment,
            "The pseudo-noise is deterministic and illustrative only; it is not a statistical validation study.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "noisy_widening".to_string(),
            mode: EnvelopeMode::Widening,
            base_radius: 0.30,
            slope: 0.0010,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_magnitude_matched_admissible() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::MagnitudeMatchedAdmissible,
        record: record(
            "magnitude_matched_admissible",
            "Magnitude-Matched Admissible Pair Member",
            "Provide a same-scale reference trajectory for the detectability-is-not-magnitude-alone comparison.",
            "Residual magnitude is intentionally similar to the detectable counterpart while evolution remains bounded and largely grammatical.",
            ClaimClass::TheoremAligned,
            "The pair demonstrates directional dependence under a shared construction; it is not a statement about all same-magnitude trajectories.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "magnitude_pair_fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 0.43,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: None,
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn scenario_magnitude_matched_detectable() -> ScenarioDefinition {
    ScenarioDefinition {
        kind: ScenarioKind::MagnitudeMatchedDetectable,
        record: record(
            "magnitude_matched_detectable",
            "Magnitude-Matched Detectable Pair Member",
            "Contrast with the admissible pair member to show that detectability depends on relative evolution, not magnitude alone.",
            "The trajectory is built to have similar residual scale but a persistent outward trend that eventually breaks admissibility.",
            ClaimClass::TheoremAligned,
            "The comparison is controlled and synthetic; it demonstrates a sufficient-condition contrast rather than a universal dichotomy.",
        ),
        channels: channels(3),
        envelope_spec: EnvelopeSpec {
            name: "magnitude_pair_widening".to_string(),
            mode: EnvelopeMode::Widening,
            base_radius: 0.43,
            slope: 0.0004,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        detectability_inputs: Some(DetectabilityBoundInputs {
            t0: 0.0,
            alpha: 0.0028,
            kappa: 0.0004,
            delta0: 0.19,
        }),
        groups: Vec::new(),
        aggregate_envelope_spec: None,
    }
}

fn record(
    id: &str,
    title: &str,
    purpose: &str,
    theorem_alignment: &str,
    claim_class: ClaimClass,
    limitations: &str,
) -> ScenarioRecord {
    ScenarioRecord {
        id: id.to_string(),
        title: title.to_string(),
        purpose: purpose.to_string(),
        theorem_alignment: theorem_alignment.to_string(),
        claim_class: claim_class.as_str().to_string(),
        limitations: limitations.to_string(),
    }
}

fn channels(count: usize) -> Vec<String> {
    (0..count).map(|index| format!("ch{}", index + 1)).collect()
}
