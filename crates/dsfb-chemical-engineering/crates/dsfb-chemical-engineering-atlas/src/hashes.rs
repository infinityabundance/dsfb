//! Deterministic SHA-256 atlas hashes.
//!
//! Each hash is built with [`crate::hashing::CanonicalHasher`] over the `const` record tables in a
//! fixed field order (records are already stored sorted by their stable key). The digests are pure
//! functions of the bundled authority data: identical across runs and machines, with a frozen
//! expected-hex test pinning them. They let a downstream court (the `cuda` crate) seal *which* atlas
//! version a case file was interpreted against.

use alloc::string::String;

use crate::detector::{
    DeterministicStatus, FusionAxis, ImplementationStatus, NegativeWitnessKind, OutputWitnessKind,
    PrimitiveFamily, WitnessRole, DETECTOR_RECORDS,
};
use crate::fault_signature::{FaultMechanism, FAULT_SIGNATURES};
use crate::hashing::CanonicalHasher;
use crate::heuristics::HEURISTIC_RECORDS;
use crate::unknown_taxonomy::UNKNOWN_TAXONOMY;

const fn pf(p: PrimitiveFamily) -> &'static str {
    match p {
        PrimitiveFamily::ScalarThreshold => "ScalarThreshold",
        PrimitiveFamily::ProjectionResidual => "ProjectionResidual",
        PrimitiveFamily::WindowStatistic => "WindowStatistic",
        PrimitiveFamily::SequentialRecurrence => "SequentialRecurrence",
        PrimitiveFamily::RankStatistic => "RankStatistic",
        PrimitiveFamily::DistributionDistance => "DistributionDistance",
        PrimitiveFamily::Spectral => "Spectral",
        PrimitiveFamily::ResidualObserver => "ResidualObserver",
    }
}

const fn ow(o: OutputWitnessKind) -> &'static str {
    match o {
        OutputWitnessKind::ScalarExceedance => "ScalarExceedance",
        OutputWitnessKind::LatentDistance => "LatentDistance",
        OutputWitnessKind::ResidualNorm => "ResidualNorm",
        OutputWitnessKind::TrendStatistic => "TrendStatistic",
        OutputWitnessKind::Changepoint => "Changepoint",
        OutputWitnessKind::DistributionShift => "DistributionShift",
        OutputWitnessKind::DensityDistance => "DensityDistance",
        OutputWitnessKind::SpectralShift => "SpectralShift",
        OutputWitnessKind::CoMonotoneCount => "CoMonotoneCount",
        OutputWitnessKind::StructuralIsolation => "StructuralIsolation",
    }
}

const fn nw(n: NegativeWitnessKind) -> &'static str {
    match n {
        NegativeWitnessKind::WithinControlLimit => "WithinControlLimit",
        NegativeWitnessKind::NoTrend => "NoTrend",
        NegativeWitnessKind::NoDistributionShift => "NoDistributionShift",
        NegativeWitnessKind::NoSpectralShift => "NoSpectralShift",
        NegativeWitnessKind::NoStructuralBreak => "NoStructuralBreak",
        NegativeWitnessKind::NoIsolatedDeviation => "NoIsolatedDeviation",
    }
}

const fn role(r: WitnessRole) -> &'static str {
    match r {
        WitnessRole::Primary => "Primary",
        WitnessRole::Supporting => "Supporting",
    }
}

const fn det(d: DeterministicStatus) -> &'static str {
    match d {
        DeterministicStatus::DeterministicNative => "DeterministicNative",
        DeterministicStatus::Stochastic => "Stochastic",
    }
}

const fn impl_status(i: ImplementationStatus) -> &'static str {
    match i {
        ImplementationStatus::Executed => "Executed",
        ImplementationStatus::Catalogued => "Catalogued",
    }
}

const fn ax(a: FusionAxis) -> &'static str {
    match a {
        FusionAxis::ResidualMagnitude => "ResidualMagnitude",
        FusionAxis::LatentMagnitude => "LatentMagnitude",
        FusionAxis::ResidualEnergy => "ResidualEnergy",
        FusionAxis::TemporalDrift => "TemporalDrift",
        FusionAxis::Changepoint => "Changepoint",
        FusionAxis::DistributionShift => "DistributionShift",
        FusionAxis::Density => "Density",
        FusionAxis::Spectral => "Spectral",
        FusionAxis::StructureCoherence => "StructureCoherence",
        FusionAxis::Attribution => "Attribution",
    }
}

/// SHA-256 over the detector records (fixed field order; records already sorted by `canonical_id`).
pub fn detector_atlas_hash_v1() -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"detector_atlas_v1");
    for d in DETECTOR_RECORDS {
        h.u64("canonical_id", d.canonical_id as u64);
        h.field("detector_id", d.detector_id.as_bytes());
        h.field("display_name", d.display_name.as_bytes());
        h.field("family", d.family.as_bytes());
        h.field("primitive_family", pf(d.primitive_family).as_bytes());
        h.field("mathematical_form", d.mathematical_form.as_bytes());
        h.field("decision_functional", d.decision_functional.as_bytes());
        h.field("output_witness", ow(d.output_witness).as_bytes());
        h.field("witness_role", role(d.witness_role).as_bytes());
        h.field("negative_witness", nw(d.negative_witness_kind).as_bytes());
        h.field(
            "deterministic_status",
            det(d.deterministic_status).as_bytes(),
        );
        h.field(
            "implementation_status",
            impl_status(d.implementation_status).as_bytes(),
        );
        for a in d.fusion_axes {
            h.field("fusion_axis", ax(*a).as_bytes());
        }
        for s in d.source_refs {
            h.field("source_ref", s.as_bytes());
        }
    }
    h.finalize_hex()
}

/// SHA-256 over the H1–H6 heuristic records (fixed field order; in id order).
pub fn chemical_process_heuristics_hash_v1() -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"process_heuristics_v1");
    for r in HEURISTIC_RECORDS {
        h.field("heuristic_id", r.heuristic_id.as_bytes());
        h.field("name", r.name.as_bytes());
        h.field("residual_pattern", r.residual_pattern.as_bytes());
        h.field("episode_label", r.episode_label.as_bytes());
        for d in r.detector_inputs {
            h.field("detector_input", d.as_bytes());
        }
    }
    h.finalize_hex()
}

/// SHA-256 over the per-heuristic condition docket (the inspectable drift/slew/admissibility rule).
pub fn challenge_docket_hash_v1() -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"challenge_docket_v1");
    for r in HEURISTIC_RECORDS {
        h.field("heuristic_id", r.heuristic_id.as_bytes());
        h.field("drift_condition", r.drift_condition.as_bytes());
        h.field("slew_condition", r.slew_condition.as_bytes());
        h.field(
            "admissibility_condition",
            r.admissibility_condition.as_bytes(),
        );
    }
    h.finalize_hex()
}

const fn mech(m: FaultMechanism) -> &'static str {
    match m {
        FaultMechanism::ValveStiction => "ValveStiction",
        FaultMechanism::Cavitation => "Cavitation",
        FaultMechanism::HeatTransferFouling => "HeatTransferFouling",
        FaultMechanism::HeatExchangerBypass => "HeatExchangerBypass",
        FaultMechanism::PumpBearingDegradation => "PumpBearingDegradation",
        FaultMechanism::ProcessLeak => "ProcessLeak",
        FaultMechanism::SensorDriftBias => "SensorDriftBias",
        FaultMechanism::ControllerMasking => "ControllerMasking",
        FaultMechanism::ValveHunting => "ValveHunting",
        FaultMechanism::Blockage => "Blockage",
        FaultMechanism::RotorImbalance => "RotorImbalance",
        FaultMechanism::RefrigerantCharge => "RefrigerantCharge",
    }
}

/// SHA-256 over the process-fault signature bank (fixed field order; in fault_id order).
pub fn fault_signatures_hash_v1() -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"fault_signatures_v1");
    for f in FAULT_SIGNATURES {
        h.field("fault_id", f.fault_id.as_bytes());
        h.field("name", f.name.as_bytes());
        h.field("mechanism", mech(f.mechanism).as_bytes());
        h.field("residual_motif", f.residual_motif.as_bytes());
        h.field(
            "implementation_status",
            impl_status(f.implementation_status).as_bytes(),
        );
        for d in f.detector_inputs {
            h.field("detector_input", d.as_bytes());
        }
    }
    h.finalize_hex()
}

/// Composite atlas hash: seals the four sub-hashes in a fixed order.
pub fn atlas_hash_v1() -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"atlas_v1");
    h.field(
        "detector_atlas_hash_v1",
        detector_atlas_hash_v1().as_bytes(),
    );
    h.field(
        "chemical_process_heuristics_hash_v1",
        chemical_process_heuristics_hash_v1().as_bytes(),
    );
    h.field(
        "challenge_docket_hash_v1",
        challenge_docket_hash_v1().as_bytes(),
    );
    h.field(
        "fault_signatures_hash_v1",
        fault_signatures_hash_v1().as_bytes(),
    );
    // Added P60: the 7-class unknown-disposition taxonomy joins the sealed authority.
    h.field(
        "unknown_taxonomy_hash_v1",
        unknown_taxonomy_hash_v1().as_bytes(),
    );
    h.finalize_hex()
}

/// SHA-256 over the unknown-disposition taxonomy (fixed field order, in record order).
pub fn unknown_taxonomy_hash_v1() -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"unknown_taxonomy_v1");
    for c in UNKNOWN_TAXONOMY {
        h.field("class_id", c.class_id.as_bytes());
        h.field("name", c.name.as_bytes());
        h.field("description", c.description.as_bytes());
        h.field("disposition", c.disposition.as_bytes());
    }
    h.finalize_hex()
}
