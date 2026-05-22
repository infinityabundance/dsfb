//! Implementation kind — the GPU kernel shape a detector maps to.
//!
//! At S1.1 this is purely a schema tag. The S1.4 family-kernel
//! mapping consumes it to choose a launch geometry. **Default is
//! NOT a GPU claim.** A spec carrying `ImplementationKind::ScalarCpu`
//! states the kernel surface is scalar / single-thread per cell;
//! the L-band (corpus crate's [`dsfb_gpu_atlas_corpus::types::ImplementationLevel`])
//! is the authority on whether the GPU surface actually exists.
//!
//! The verifier checks the natural compatibility ordering:
//! `ScalarCpu` < `CellParallel` < `SegmentScan` for the
//! `implementation_kind_is_not_gpu_claim_by_default` acceptance
//! test.

/// GPU-kernel implementation kind. Schema-level tag; not a GPU
/// claim by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImplementationKind {
    /// Single-thread scalar CPU implementation. The default at
    /// S1.1 for any newly-declared spec; an explicit upgrade is
    /// required for cell-parallel / segment-scan.
    ScalarCpu,
    /// Cell-parallel — one GPU thread per detector cell.
    CellParallel,
    /// Per-entity segmented scan (e.g. EWMA recurrence) followed
    /// by cell-parallel evaluation.
    SegmentScan,
    /// Family-kernel — one CUDA kernel per detector family, with
    /// per-family parameter table in constant or global memory.
    FamilyKernel,
}

impl ImplementationKind {
    /// The default implementation kind for newly-declared specs.
    /// Reflects the audit-first doctrine: every detector starts
    /// as a scalar CPU implementation; faster surfaces require
    /// explicit attestation.
    pub const DEFAULT: ImplementationKind = ImplementationKind::ScalarCpu;

    /// True if the kind implies a GPU surface. Returns `false`
    /// for `ScalarCpu`; `true` for the other kinds. The
    /// `implementation_kind_is_not_gpu_claim_by_default`
    /// acceptance test pins that `DEFAULT.is_gpu_claim() ==
    /// false`.
    #[must_use]
    pub const fn is_gpu_claim(self) -> bool {
        !matches!(self, Self::ScalarCpu)
    }

    /// Canonical wire name, uppercase snake-case.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::ScalarCpu => "SCALAR_CPU",
            Self::CellParallel => "CELL_PARALLEL",
            Self::SegmentScan => "SEGMENT_SCAN",
            Self::FamilyKernel => "FAMILY_KERNEL",
        }
    }
}
