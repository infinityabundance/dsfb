use crate::grammar::{GrammarReason, GrammarSet};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicEntry {
    pub motif_name: String,
    pub signature_definition: String,
    pub applicable_dataset: String,
    pub provenance_status: String,
    pub interpretation: String,
    pub status_note: String,
}

pub fn build_heuristics_bank(grammar: &GrammarSet, dataset_name: &str) -> Vec<HeuristicEntry> {
    let mut sustained_count = 0usize;
    let mut abrupt_count = 0usize;
    let mut grazing_count = 0usize;

    for trace in &grammar.traces {
        for reason in &trace.reasons {
            match reason {
                GrammarReason::SustainedOutwardDrift => sustained_count += 1,
                GrammarReason::AbruptSlewViolation => abrupt_count += 1,
                GrammarReason::RecurrentBoundaryGrazing => grazing_count += 1,
                _ => {}
            }
        }
    }

    vec![
        HeuristicEntry {
            motif_name: "pre_failure_slow_drift".into(),
            signature_definition:
                "Residual norm exceeds 0.5*rho with drift above the healthy-window drift threshold."
                    .into(),
            applicable_dataset: dataset_name.into(),
            provenance_status: if sustained_count > 0 {
                "SECOM-observed".into()
            } else {
                "framework-defined".into()
            },
            interpretation:
                "Candidate early-warning drift motif that supports closer monitoring or maintenance review."
                    .into(),
            status_note: format!("Observed {sustained_count} boundary points with SustainedOutwardDrift."),
        },
        HeuristicEntry {
            motif_name: "transient_excursion".into(),
            signature_definition:
                "Residual norm enters the boundary zone with slew above the healthy-window slew threshold."
                    .into(),
            applicable_dataset: dataset_name.into(),
            provenance_status: if abrupt_count > 0 {
                "SECOM-observed".into()
            } else {
                "framework-defined".into()
            },
            interpretation:
                "Compatible with transient upset or abrupt regime change, but not uniquely diagnostic."
                    .into(),
            status_note: format!("Observed {abrupt_count} boundary points with AbruptSlewViolation."),
        },
        HeuristicEntry {
            motif_name: "recurrent_boundary_approach".into(),
            signature_definition:
                "Residual norm revisits the boundary zone repeatedly without a confirmed envelope exit."
                    .into(),
            applicable_dataset: dataset_name.into(),
            provenance_status: if grazing_count > 0 {
                "SECOM-observed".into()
            } else {
                "framework-defined".into()
            },
            interpretation:
                "Ambiguous precursor motif that warrants continued observation rather than decisive attribution."
                    .into(),
            status_note: format!("Observed {grazing_count} boundary points with RecurrentBoundaryGrazing."),
        },
    ]
}
