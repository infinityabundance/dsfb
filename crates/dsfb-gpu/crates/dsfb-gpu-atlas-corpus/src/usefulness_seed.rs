// usefulness_seed.rs — conservative T.8 ledger seed.
//
// One ledger row per canonical detector (54 rows total). Every
// empirical field is zero. The seed marks:
//
// - Canonical IDs 14, 15, 41, 42, 43 (the dsfb-gpu-debug-core
//   bank surface) as `RoleSeeded` + `Active` +
//   `GpuSurfaceSeededFromDsfbGpuDebugCore` +
//   `DsfbGpuDebugCoreSurface`. These rows are honest claims that
//   a GPU surface exists for the detector; they are NOT claims
//   about measured usefulness.
//
// - Every other canonical ID as `LiteraturePrior` + `Dormant` +
//   `LiteraturePriorOnly` + `AtlasCorpusSeedV1`. These rows are
//   honest claims that the detector is cited in the literature;
//   they are NOT claims about measured usefulness or about
//   activity in the current bank.
//
// Domain bitset selection: the lowest-bit set in each detector's
// `origin_domains`. This is deterministic and reviewable.
//
// `score_kind` is `NotScored` on every row. T.9+ may upgrade
// individual rows to `PriorScore` / `MeasuredScore` when a real
// benchmark artifact backs them.

/// One ledger row per canonical detector — conservative T.8
/// seed. **No empirical usefulness is claimed.** Five rows
/// (canonical IDs 14, 15, 41, 42, 43) are at `RoleSeeded` /
/// `Active` because the dsfb-gpu-debug-core bank surface
/// implements them; the remaining 49 rows are at
/// `LiteraturePrior` / `Dormant`. Every row carries
/// `score_kind = NotScored`.
pub static USEFULNESS_LEDGER: &[UsefulnessLedgerRow] = &[
    literature_dormant_row(1, DomainTagSet::TABULAR),
    literature_dormant_row(2, DomainTagSet::TELEMETRY),
    literature_dormant_row(3, DomainTagSet::TELEMETRY),
    literature_dormant_row(4, DomainTagSet::TELEMETRY),
    literature_dormant_row(5, DomainTagSet::TABULAR),
    literature_dormant_row(6, DomainTagSet::TELEMETRY),
    literature_dormant_row(7, DomainTagSet::TIME_SERIES),
    literature_dormant_row(8, DomainTagSet::TELEMETRY),
    literature_dormant_row(9, DomainTagSet::TABULAR),
    literature_dormant_row(10, DomainTagSet::TABULAR),
    literature_dormant_row(11, DomainTagSet::TIME_SERIES),
    literature_dormant_row(12, DomainTagSet::TIME_SERIES),
    literature_dormant_row(13, DomainTagSet::TABULAR),
    gpu_seeded_row(14, DomainTagSet::DEBUG),
    gpu_seeded_row(15, DomainTagSet::DEBUG),
    literature_dormant_row(16, DomainTagSet::TABULAR),
    literature_dormant_row(17, DomainTagSet::TABULAR),
    literature_dormant_row(18, DomainTagSet::TABULAR),
    literature_dormant_row(19, DomainTagSet::TABULAR),
    literature_dormant_row(20, DomainTagSet::TABULAR),
    literature_dormant_row(21, DomainTagSet::INDUSTRIAL),
    literature_dormant_row(22, DomainTagSet::TIME_SERIES),
    literature_dormant_row(23, DomainTagSet::TELEMETRY),
    literature_dormant_row(24, DomainTagSet::TIME_SERIES),
    literature_dormant_row(25, DomainTagSet::TIME_SERIES),
    literature_dormant_row(26, DomainTagSet::TABULAR),
    literature_dormant_row(27, DomainTagSet::TABULAR),
    literature_dormant_row(28, DomainTagSet::TELEMETRY),
    literature_dormant_row(29, DomainTagSet::TABULAR),
    literature_dormant_row(30, DomainTagSet::TABULAR),
    literature_dormant_row(31, DomainTagSet::TELEMETRY),
    literature_dormant_row(32, DomainTagSet::TABULAR),
    literature_dormant_row(33, DomainTagSet::TABULAR),
    literature_dormant_row(34, DomainTagSet::TELEMETRY),
    literature_dormant_row(35, DomainTagSet::TIME_SERIES),
    literature_dormant_row(36, DomainTagSet::TELEMETRY),
    literature_dormant_row(37, DomainTagSet::TIME_SERIES),
    literature_dormant_row(38, DomainTagSet::TIME_SERIES),
    literature_dormant_row(39, DomainTagSet::TIME_SERIES),
    literature_dormant_row(40, DomainTagSet::TELEMETRY),
    gpu_seeded_row(41, DomainTagSet::DEBUG),
    gpu_seeded_row(42, DomainTagSet::DEBUG),
    gpu_seeded_row(43, DomainTagSet::DEBUG),
    literature_dormant_row(44, DomainTagSet::TABULAR),
    literature_dormant_row(45, DomainTagSet::TABULAR),
    literature_dormant_row(46, DomainTagSet::TABULAR),
    literature_dormant_row(47, DomainTagSet::TABULAR),
    literature_dormant_row(48, DomainTagSet::TABULAR),
    literature_dormant_row(49, DomainTagSet::TIME_SERIES),
    literature_dormant_row(50, DomainTagSet::TIME_SERIES),
    literature_dormant_row(51, DomainTagSet::TIME_SERIES),
    literature_dormant_row(52, DomainTagSet::TIME_SERIES),
    literature_dormant_row(53, DomainTagSet::TIME_SERIES),
    literature_dormant_row(54, DomainTagSet::TIME_SERIES),
];

// Helper constructors are `const fn` so the seed is a true `pub
// static` with no runtime initialisation. Two builds produce
// byte-identical USEFULNESS_LEDGER bytes.

const fn literature_dormant_row(canonical_id: u32, domain_bit: u16) -> UsefulnessLedgerRow {
    UsefulnessLedgerRow {
        canonical_id: DetectorCanonicalId(canonical_id),
        task_id: SEED_TASK_ID,
        domain: DomainTagSet(domain_bit),
        dataset_id: SEED_DATASET_ID,
        evidence_level: UsefulnessEvidenceLevel::LiteraturePrior,
        lifecycle_state: LifecycleState::Dormant,
        score_kind: UsefulnessScoreKind::NotScored,
        unique_episode_gain: 0,
        redundant_with_count: 0,
        clean_window_false_positive_cost: 0,
        confuser_reduction_gain: 0,
        runtime_cost_us_p50: 0,
        memory_cost_bytes: 0,
        casefile_explanation_value: 0,
        operator_readability_score: 0,
        sample_count: 0,
        ledger_source: LedgerSource::AtlasCorpusSeedV1,
        reason_code: UsefulnessReason::LiteraturePriorOnly,
    }
}

const fn gpu_seeded_row(canonical_id: u32, domain_bit: u16) -> UsefulnessLedgerRow {
    UsefulnessLedgerRow {
        canonical_id: DetectorCanonicalId(canonical_id),
        task_id: SEED_TASK_ID,
        domain: DomainTagSet(domain_bit),
        dataset_id: SEED_DATASET_ID,
        evidence_level: UsefulnessEvidenceLevel::RoleSeeded,
        lifecycle_state: LifecycleState::Active,
        score_kind: UsefulnessScoreKind::NotScored,
        unique_episode_gain: 0,
        redundant_with_count: 0,
        clean_window_false_positive_cost: 0,
        confuser_reduction_gain: 0,
        runtime_cost_us_p50: 0,
        memory_cost_bytes: 0,
        casefile_explanation_value: 0,
        operator_readability_score: 0,
        sample_count: 0,
        ledger_source: LedgerSource::DsfbGpuDebugCoreSurface,
        reason_code: UsefulnessReason::GpuSurfaceSeededFromDsfbGpuDebugCore,
    }
}
