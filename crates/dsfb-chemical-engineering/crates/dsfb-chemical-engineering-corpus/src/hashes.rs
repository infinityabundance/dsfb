//! Deterministic SHA-256 corpus hash over the soft-sensor dataset catalogue.

use alloc::string::String;

use crate::hashing::CanonicalHasher;
use crate::seed::SOFT_SENSOR_DATASETS;
use crate::types::{
    AccessConfidence, LicenseConfidence, ProcessDomain, RedistributionPolicy, SourceAuthorityKind,
    TargetKind,
};

const fn dom(d: ProcessDomain) -> &'static str {
    match d {
        ProcessDomain::RefineryDistillation => "RefineryDistillation",
        ProcessDomain::MineralProcessing => "MineralProcessing",
        ProcessDomain::Power => "Power",
        ProcessDomain::Wastewater => "Wastewater",
        ProcessDomain::Bioprocess => "Bioprocess",
        ProcessDomain::MultiphaseFlow => "MultiphaseFlow",
        ProcessDomain::Semiconductor => "Semiconductor",
        ProcessDomain::GasSensing => "GasSensing",
        ProcessDomain::PulpPaper => "PulpPaper",
        ProcessDomain::Emissions => "Emissions",
        ProcessDomain::Food => "Food",
        ProcessDomain::GeneralProcess => "GeneralProcess",
    }
}

const fn tk(t: TargetKind) -> &'static str {
    match t {
        TargetKind::Composition => "Composition",
        TargetKind::Concentration => "Concentration",
        TargetKind::Quality => "Quality",
        TargetKind::Emissions => "Emissions",
        TargetKind::Energy => "Energy",
        TargetKind::RemovalEfficiency => "RemovalEfficiency",
        TargetKind::FaultLabel => "FaultLabel",
    }
}

// Stable string tags for the P53 classification tiers. These names are part of the sealed
// `corpus_hash_v1` preimage — renaming a tag deliberately re-freezes the hash (a governed change),
// so the tags are fixed and decoupled from any future Display formatting.
const fn lic(l: LicenseConfidence) -> &'static str {
    match l {
        LicenseConfidence::ExplicitOpen => "ExplicitOpen",
        LicenseConfidence::ExplicitCopyleft => "ExplicitCopyleft",
        LicenseConfidence::StatedNeedsVerification => "StatedNeedsVerification",
        LicenseConfidence::ResearchUseCustomary => "ResearchUseCustomary",
        LicenseConfidence::AgreementGoverned => "AgreementGoverned",
    }
}

const fn acc(a: AccessConfidence) -> &'static str {
    match a {
        AccessConfidence::OpenConfirmed => "OpenConfirmed",
        AccessConfidence::OpenMirrorUnverified => "OpenMirrorUnverified",
        AccessConfidence::AccountRequired => "AccountRequired",
        AccessConfidence::GeneratedByCode => "GeneratedByCode",
        AccessConfidence::AgreementRequired => "AgreementRequired",
    }
}

const fn redist(r: RedistributionPolicy) -> &'static str {
    match r {
        RedistributionPolicy::UpstreamPermitsAttribution => "UpstreamPermitsAttribution",
        RedistributionPolicy::UpstreamCopyleftShareAlike => "UpstreamCopyleftShareAlike",
        RedistributionPolicy::UpstreamVerifyBeforeRedistribution => {
            "UpstreamVerifyBeforeRedistribution"
        }
        RedistributionPolicy::ProhibitedByAgreement => "ProhibitedByAgreement",
    }
}

const fn auth(s: SourceAuthorityKind) -> &'static str {
    match s {
        SourceAuthorityKind::DoiArchive => "DoiArchive",
        SourceAuthorityKind::CuratedMlRepository => "CuratedMlRepository",
        SourceAuthorityKind::PackageDistribution => "PackageDistribution",
        SourceAuthorityKind::SimulatorCodebase => "SimulatorCodebase",
        SourceAuthorityKind::GovernedTestbed => "GovernedTestbed",
        SourceAuthorityKind::CommunityUpload => "CommunityUpload",
        SourceAuthorityKind::AuthorOrVendorHost => "AuthorOrVendorHost",
    }
}

/// SHA-256 over the soft-sensor dataset catalogue (fixed field order; in seed order).
///
/// The P53 classification tiers (`license_confidence`, `access_confidence`, `redistribution_policy`,
/// `source_authority`) are folded in after the source fields, so the sealed authority now includes the
/// provenance classification. Adding them re-froze `corpus_hash_v1` once (a governed, documented
/// change); the gate in `tests/corpus_gates.rs` pins the new value.
pub fn corpus_hash_v1() -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"soft_sensor_corpus_v1");
    for d in SOFT_SENSOR_DATASETS {
        h.field("dataset_id", d.dataset_id.as_bytes());
        h.field("name", d.name.as_bytes());
        h.field("domain", dom(d.domain).as_bytes());
        h.field("target", d.target.as_bytes());
        h.field("target_kind", tk(d.target_kind).as_bytes());
        h.u64("n_cheap_inputs", d.n_cheap_inputs as u64);
        h.u64("cheap_sensor", d.cheap_sensor as u64);
        h.field("citation_key", d.source.citation_key.as_bytes());
        h.field("url", d.source.url.as_bytes());
        h.field("license_confidence", lic(d.license_confidence).as_bytes());
        h.field("access_confidence", acc(d.access_confidence).as_bytes());
        h.field(
            "redistribution_policy",
            redist(d.redistribution_policy).as_bytes(),
        );
        h.field("source_authority", auth(d.source_authority).as_bytes());
    }
    h.finalize_hex()
}
