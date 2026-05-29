//! Deterministic validation gates over the soft-sensor dataset catalogue.
//!
//! Pure functions of the `const` seed table; errors are `&'static str`. Enforces the provenance
//! discipline (every dataset sourced + access-flagged + never redistributed) and the canonical core.

use crate::seed::SOFT_SENSOR_DATASETS;
use crate::types::{
    AccessConfidence, LicenseConfidence, ProcessDomain, RedistributionPolicy, SourceAuthorityKind,
};

/// Summary of a passing corpus validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CorpusValidationReport {
    pub n_datasets: usize,
    pub n_cheap_sensor: usize,
    pub n_with_fault_labels: usize,
    pub n_domains: usize,
}

/// Run every gate. Returns the summary report or the first failing gate's message.
pub fn validate() -> Result<CorpusValidationReport, &'static str> {
    let ds = SOFT_SENSOR_DATASETS;
    if ds.is_empty() {
        return Err("empty soft-sensor catalogue");
    }

    for (i, d) in ds.iter().enumerate() {
        if d.dataset_id.is_empty() || d.target.is_empty() {
            return Err("dataset missing dataset_id / target");
        }
        if d.cheap_sensors.is_empty() {
            return Err("dataset missing sensor channels");
        }
        // Provenance discipline: every record is sourced and never redistributes bytes.
        if d.source.citation_key.is_empty()
            || d.source.url.is_empty()
            || d.source.license.is_empty()
        {
            return Err("dataset missing source citation / url / licence");
        }
        if d.redistributed {
            return Err("dataset bytes must NEVER be redistributed by this crate");
        }
        if d.deterministic_inference.is_empty() {
            return Err("dataset missing deterministic_inference note");
        }
        for e in &ds[i + 1..] {
            if d.dataset_id == e.dataset_id {
                return Err("duplicate dataset_id");
            }
        }
    }

    // Canonical soft-sensor benchmarks must be present.
    for id in ["sru", "debutanizer", "tep", "mining_flotation", "ccpp"] {
        if !ds.iter().any(|d| d.dataset_id == id) {
            return Err("a canonical soft-sensor benchmark is missing");
        }
    }

    // Distinct domain spread (a broad chemical-engineering surface).
    let mut n_domains = 0usize;
    for (i, d) in ds.iter().enumerate() {
        if !ds[..i].iter().any(|e| same_domain(e.domain, d.domain)) {
            n_domains += 1;
        }
    }
    if n_domains < 6 {
        return Err("expected a broad process-domain spread (>= 6)");
    }

    Ok(CorpusValidationReport {
        n_datasets: ds.len(),
        n_cheap_sensor: ds.iter().filter(|d| d.cheap_sensor).count(),
        n_with_fault_labels: ds.iter().filter(|d| d.has_fault_labels).count(),
        n_domains,
    })
}

fn same_domain(a: ProcessDomain, b: ProcessDomain) -> bool {
    a == b
}

/// A per-tier tally of the four P53 provenance-classification axes (license / access / redistribution
/// / source authority). It is a pure **disclosure** count over the seed table — it never fails a build
/// or asserts a quality judgement; it simply makes the catalogue's licence/access posture auditable at
/// a glance (and lets a gate confirm every axis partitions all records). One `usize` per tier; the
/// per-axis subtotals each sum to `n_datasets`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassificationCensus {
    pub n_datasets: usize,
    // license_confidence
    pub lic_explicit_open: usize,
    pub lic_explicit_copyleft: usize,
    pub lic_stated_needs_verification: usize,
    pub lic_research_use_customary: usize,
    pub lic_agreement_governed: usize,
    // access_confidence
    pub acc_open_confirmed: usize,
    pub acc_open_mirror_unverified: usize,
    pub acc_account_required: usize,
    pub acc_generated_by_code: usize,
    pub acc_agreement_required: usize,
    // redistribution_policy
    pub redist_permits_attribution: usize,
    pub redist_copyleft_share_alike: usize,
    pub redist_verify_before: usize,
    pub redist_prohibited_by_agreement: usize,
    // source_authority
    pub auth_doi_archive: usize,
    pub auth_curated_ml_repository: usize,
    pub auth_package_distribution: usize,
    pub auth_simulator_codebase: usize,
    pub auth_governed_testbed: usize,
    pub auth_community_upload: usize,
    pub auth_author_or_vendor_host: usize,
}

impl ClassificationCensus {
    /// Subtotal across the five `license_confidence` tiers (must equal `n_datasets`).
    pub fn license_total(&self) -> usize {
        self.lic_explicit_open
            + self.lic_explicit_copyleft
            + self.lic_stated_needs_verification
            + self.lic_research_use_customary
            + self.lic_agreement_governed
    }
    /// Subtotal across the five `access_confidence` tiers (must equal `n_datasets`).
    pub fn access_total(&self) -> usize {
        self.acc_open_confirmed
            + self.acc_open_mirror_unverified
            + self.acc_account_required
            + self.acc_generated_by_code
            + self.acc_agreement_required
    }
    /// Subtotal across the four `redistribution_policy` tiers (must equal `n_datasets`).
    pub fn redistribution_total(&self) -> usize {
        self.redist_permits_attribution
            + self.redist_copyleft_share_alike
            + self.redist_verify_before
            + self.redist_prohibited_by_agreement
    }
    /// Subtotal across the seven `source_authority` tiers (must equal `n_datasets`).
    pub fn authority_total(&self) -> usize {
        self.auth_doi_archive
            + self.auth_curated_ml_repository
            + self.auth_package_distribution
            + self.auth_simulator_codebase
            + self.auth_governed_testbed
            + self.auth_community_upload
            + self.auth_author_or_vendor_host
    }
}

/// Count every classification tier across the seed table. Disclosure only — never fails.
pub fn census() -> ClassificationCensus {
    let mut c = ClassificationCensus {
        n_datasets: SOFT_SENSOR_DATASETS.len(),
        ..Default::default()
    };
    for d in SOFT_SENSOR_DATASETS {
        match d.license_confidence {
            LicenseConfidence::ExplicitOpen => c.lic_explicit_open += 1,
            LicenseConfidence::ExplicitCopyleft => c.lic_explicit_copyleft += 1,
            LicenseConfidence::StatedNeedsVerification => c.lic_stated_needs_verification += 1,
            LicenseConfidence::ResearchUseCustomary => c.lic_research_use_customary += 1,
            LicenseConfidence::AgreementGoverned => c.lic_agreement_governed += 1,
        }
        match d.access_confidence {
            AccessConfidence::OpenConfirmed => c.acc_open_confirmed += 1,
            AccessConfidence::OpenMirrorUnverified => c.acc_open_mirror_unverified += 1,
            AccessConfidence::AccountRequired => c.acc_account_required += 1,
            AccessConfidence::GeneratedByCode => c.acc_generated_by_code += 1,
            AccessConfidence::AgreementRequired => c.acc_agreement_required += 1,
        }
        match d.redistribution_policy {
            RedistributionPolicy::UpstreamPermitsAttribution => c.redist_permits_attribution += 1,
            RedistributionPolicy::UpstreamCopyleftShareAlike => c.redist_copyleft_share_alike += 1,
            RedistributionPolicy::UpstreamVerifyBeforeRedistribution => c.redist_verify_before += 1,
            RedistributionPolicy::ProhibitedByAgreement => c.redist_prohibited_by_agreement += 1,
        }
        match d.source_authority {
            SourceAuthorityKind::DoiArchive => c.auth_doi_archive += 1,
            SourceAuthorityKind::CuratedMlRepository => c.auth_curated_ml_repository += 1,
            SourceAuthorityKind::PackageDistribution => c.auth_package_distribution += 1,
            SourceAuthorityKind::SimulatorCodebase => c.auth_simulator_codebase += 1,
            SourceAuthorityKind::GovernedTestbed => c.auth_governed_testbed += 1,
            SourceAuthorityKind::CommunityUpload => c.auth_community_upload += 1,
            SourceAuthorityKind::AuthorOrVendorHost => c.auth_author_or_vendor_host += 1,
        }
    }
    c
}
