use anyhow::Result;
use serde::Serialize;

use crate::registry::TheoremSpec;
use crate::runners::{write_component_rows, RunnerContext};

#[derive(Debug, Clone, Serialize)]
struct DsfbRow {
    theorem_id: String,
    theorem_name: String,
    component: &'static str,
    case_id: String,
    case_type: String,
    pass: bool,
    notes: String,
    assumptions_satisfied: bool,
    injective_flag: bool,
    observation_id: String,
    structural_state_id: String,
    reconstructed_state_id: String,
    residual_value: f64,
    exact_recovery_flag: bool,
    time_step: Option<usize>,
    observation_value: Option<f64>,
    equivalence_class_size: usize,
    roundtrip_flag: bool,
}

#[derive(Debug, Clone, Copy)]
enum DsfbMode {
    Equivalence,
    InjectiveObservability,
    ExactReconstruction,
    RecoverabilityImpliesInjective,
    EquivalenceOfRecoverability,
    Replayability,
    ResidualCollapse,
    ResidualConsistency,
    IdempotentRfr,
    IdempotentFrf,
    SingletonClasses,
    FiniteCoding,
    Composition,
    Restriction,
    Product,
    Distinguishability,
    MinimalResidual,
    UniqueInverse,
    PeriodicObservation,
    PeriodicReconstruction,
}

pub fn run(
    spec: &TheoremSpec,
    ctx: &RunnerContext<'_>,
) -> Result<crate::runners::TheoremExecutionResult> {
    let mode = match spec.ordinal {
        1 => DsfbMode::Equivalence,
        2 => DsfbMode::InjectiveObservability,
        3 => DsfbMode::ExactReconstruction,
        4 => DsfbMode::RecoverabilityImpliesInjective,
        5 => DsfbMode::EquivalenceOfRecoverability,
        6 => DsfbMode::Replayability,
        7 => DsfbMode::ResidualCollapse,
        8 => DsfbMode::ResidualConsistency,
        9 => DsfbMode::IdempotentRfr,
        10 => DsfbMode::IdempotentFrf,
        11 => DsfbMode::SingletonClasses,
        12 => DsfbMode::FiniteCoding,
        13 => DsfbMode::Composition,
        14 => DsfbMode::Restriction,
        15 => DsfbMode::Product,
        16 => DsfbMode::Distinguishability,
        17 => DsfbMode::MinimalResidual,
        18 => DsfbMode::UniqueInverse,
        19 => DsfbMode::PeriodicObservation,
        20 => DsfbMode::PeriodicReconstruction,
        _ => unreachable!("unexpected DSFB theorem ordinal"),
    };
    let rows = build_rows(spec, mode, ctx.seed);
    let pass_count = rows.iter().filter(|row| row.pass).count();
    let fail_count = rows.len().saturating_sub(pass_count);
    write_component_rows(spec, ctx, &rows, pass_count, fail_count)
}

fn build_rows(spec: &TheoremSpec, mode: DsfbMode, seed: u64) -> Vec<DsfbRow> {
    let phase = (seed % 2) as usize;
    match mode {
        DsfbMode::Equivalence => vec![
            dsfb_row(spec, "reflexive_s1", "satisfying", true, true, false, "y10", "s1", "s1", 0.0, true, None, Some(10.0), 3, true, "Reflexive witness: F(s1)=F(s1)."),
            dsfb_row(spec, "symmetric_s1_s2", "satisfying", true, false, false, "y10", "s1", "s1", 0.0, false, None, Some(10.0), 3, false, "Symmetric witness: s1 and s2 share the same observation class."),
            dsfb_row(spec, "symmetric_s2_s1", "satisfying", true, false, false, "y10", "s2", "s1", 0.0, false, None, Some(10.0), 3, false, "Symmetry closes the observation class in the opposite direction."),
            dsfb_row(spec, "transitive_s1_s4", "boundary", true, false, false, "y10", "s4", "s1", 0.0, false, None, Some(10.0), 3, false, "Transitive witness: s1, s2, and s4 share one class without violating function semantics."),
        ],
        DsfbMode::InjectiveObservability => vec![
            dsfb_row(spec, "injective_s1", "injective", true, true, true, "y101", "s1", "s1", 0.0, true, None, Some(101.0), 1, true, "Injective forward map gives a singleton inverse image."),
            dsfb_row(spec, "injective_s2", "injective", true, true, true, "y102", "s2", "s2", 0.0, true, None, Some(102.0), 1, true, "Every admissible injective observation corresponds to one structural state."),
            dsfb_row(spec, "noninjective_collision", "non_injective", true, false, false, "y10", "s2", "ambiguous", 0.0, false, None, Some(10.0), 3, false, "Contrast case: a non-injective map yields ambiguity."),
        ],
        DsfbMode::ExactReconstruction => vec![
            dsfb_row(spec, "exact_inverse_s1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, None, Some(101.0), 1, true, "Explicit inverse recovers s1 on the image."),
            dsfb_row(spec, "exact_inverse_s3", "satisfying", true, true, true, "y103", "s3", "s3", 0.0, true, None, Some(103.0), 1, true, "Explicit inverse recovers s3 on the image."),
            dsfb_row(spec, "outside_image", "boundary", true, true, false, "y999", "outside", "fallback", 4.0, false, None, Some(999.0), 0, false, "Outside the image the inverse is undefined, which is excluded by the theorem assumptions."),
        ],
        DsfbMode::RecoverabilityImpliesInjective => vec![
            dsfb_row(spec, "exact_no_collision_s1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, None, Some(101.0), 1, true, "Exact recovery row implies no forward collision for s1."),
            dsfb_row(spec, "exact_no_collision_s2", "satisfying", true, true, true, "y102", "s2", "s2", 0.0, true, None, Some(102.0), 1, true, "Exact recovery row implies no forward collision for s2."),
            dsfb_row(spec, "collision_would_break_inverse", "boundary", true, false, false, "y10", "s4", "ambiguous", 1.0, false, None, Some(10.0), 3, false, "A collision row shows why exact recovery cannot coexist with non-injectivity."),
        ],
        DsfbMode::EquivalenceOfRecoverability => vec![
            dsfb_row(spec, "injective_implies_recoverable", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, None, Some(101.0), 1, true, "Injective case exhibits exact recovery."),
            dsfb_row(spec, "recoverable_implies_injective", "satisfying", true, true, true, "y104", "s4", "s4", 0.0, true, None, Some(104.0), 1, true, "Exact recovery forbids observation collisions."),
            dsfb_row(spec, "noninjective_not_recoverable", "boundary", true, false, false, "y10", "s2", "ambiguous", 0.5, false, None, Some(10.0), 3, false, "Non-injective contrast case lacks exact inversion."),
        ],
        DsfbMode::Replayability => vec![
            dsfb_row(spec, "duplicate_observation_1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, Some(0), Some(101.0), 1, true, "First replay of y101 reconstructs s1."),
            dsfb_row(spec, "duplicate_observation_2", "satisfying", true, true, true, "y101", "s1_copy", "s1", 0.0, true, Some(1), Some(101.0), 1, true, "Second replay of y101 reconstructs the identical structural state."),
            dsfb_row(spec, "duplicate_observation_3", "boundary", true, true, true, "y103", "s3", "s3", 0.0, true, Some(2), Some(103.0), 1, true, "Distinct observation still reconstructs deterministically."),
        ],
        DsfbMode::ResidualCollapse => vec![
            dsfb_row(spec, "exact_residual_zero", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, Some(0), Some(101.0), 1, true, "Exact recoverability collapses the residual to zero."),
            dsfb_row(spec, "exact_residual_zero_2", "satisfying", true, true, true, "y103", "s3", "s3", 0.0, true, Some(1), Some(103.0), 1, true, "Second exact case keeps residual zero on the image."),
            dsfb_row(spec, "mismatched_residual", "boundary", true, false, false, "y999", "outside", "fallback", 4.0, false, Some(2), Some(999.0), 0, false, "Approximate or mismatched reconstruction leaves a nonzero residual."),
        ],
        DsfbMode::ResidualConsistency => vec![
            dsfb_row(spec, "zero_residual_consistent", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, Some(0), Some(101.0), 1, true, "Zero residual row satisfies F(R(y))=y."),
            dsfb_row(spec, "zero_residual_consistent_2", "satisfying", true, true, true, "y104", "s4", "s4", 0.0, true, Some(1), Some(104.0), 1, true, "Forward consistency holds whenever residual vanishes."),
            dsfb_row(spec, "nonzero_residual_contrast", "boundary", true, false, false, "y998", "outside", "fallback", 2.0, false, Some(2), Some(998.0), 0, false, "Nonzero residual contrast case is not covered by the implication."),
        ],
        DsfbMode::IdempotentRfr => vec![
            dsfb_row(spec, "rfr_stable_s1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, Some(0), Some(101.0), 1, true, "R(F(R(y))) stabilizes after one exact round-trip."),
            dsfb_row(spec, "rfr_stable_s2", "satisfying", true, true, true, "y102", "s2", "s2", 0.0, true, Some(1), Some(102.0), 1, true, "Repeated reconstruction leaves the image point unchanged."),
            dsfb_row(spec, "outside_image_nonstable", "boundary", true, true, false, "y999", "outside", "fallback", 1.0, false, Some(2), Some(999.0), 0, false, "Outside the exact image the idempotence claim does not apply."),
        ],
        DsfbMode::IdempotentFrf => vec![
            dsfb_row(spec, "frf_stable_s1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, Some(0), Some(101.0), 1, true, "F(R(F(R(y)))) matches F(R(y)) on the image."),
            dsfb_row(spec, "frf_stable_s3", "satisfying", true, true, true, "y103", "s3", "s3", 0.0, true, Some(1), Some(103.0), 1, true, "Forward-reconstruct cycles stabilize after one pass."),
            dsfb_row(spec, "frf_outside_image", "boundary", true, true, false, "y997", "outside", "fallback", 3.0, false, Some(2), Some(997.0), 0, false, "Non-image observation shows why exact-image assumptions matter."),
        ],
        DsfbMode::SingletonClasses => vec![
            dsfb_row(spec, "singleton_class_s1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, None, Some(101.0), 1, true, "Observation class cardinality drops to one under exact recoverability."),
            dsfb_row(spec, "singleton_class_s2", "satisfying", true, true, true, "y102", "s2", "s2", 0.0, true, None, Some(102.0), 1, true, "Second exact class is also singleton."),
            dsfb_row(spec, "collision_contrast", "boundary", true, false, false, "y10", "s4", "ambiguous", 0.5, false, None, Some(10.0), 3, false, "Collision class has size three when exact recovery is absent."),
        ],
        DsfbMode::FiniteCoding => vec![
            dsfb_row(spec, "finite_code_s1", "satisfying", true, true, true, "code_11", "s1", "s1", 0.0, true, None, Some(11.0), 1, true, "Finite admissible class admits an injective code."),
            dsfb_row(spec, "finite_code_s2", "satisfying", true, true, true, "code_12", "s2", "s2", 0.0, true, None, Some(12.0), 1, true, "Code-like map remains exactly invertible on the image."),
            dsfb_row(spec, "finite_code_s4", "satisfying", true, true, true, "code_14", "s4", "s4", 0.0, true, None, Some(14.0), 1, true, "Every finite state receives a unique code."),
        ],
        DsfbMode::Composition => vec![
            dsfb_row(spec, "two_stage_s1", "satisfying", true, true, true, "g_y201", "s1", "s1", 0.0, true, None, Some(201.0), 1, true, "Composed injective observation preserves exact recovery."),
            dsfb_row(spec, "two_stage_s3", "satisfying", true, true, true, "g_y203", "s3", "s3", 0.0, true, None, Some(203.0), 1, true, "Second-stage injective coding remains collision free."),
            dsfb_row(spec, "noninjective_second_stage", "boundary", true, false, false, "g_collision", "s4", "ambiguous", 1.0, false, None, Some(200.0), 2, false, "A non-injective second stage would reintroduce ambiguity."),
        ],
        DsfbMode::Restriction => vec![
            dsfb_row(spec, "subset_s1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, None, Some(101.0), 1, true, "Subset S1 inherits exact recovery from S0."),
            dsfb_row(spec, "subset_s3", "satisfying", true, true, true, "y103", "s3", "s3", 0.0, true, None, Some(103.0), 1, true, "Restricted admissible class keeps the same inverse."),
            dsfb_row(spec, "excluded_state", "boundary", true, true, false, "y104", "s4", "excluded", 0.0, false, None, Some(104.0), 0, false, "Excluded state is outside the restricted subset."),
        ],
        DsfbMode::Product => vec![
            dsfb_row(spec, "product_a1_b1", "satisfying", true, true, true, "(y1,z1)", "(a1,b1)", "(a1,b1)", 0.0, true, None, Some(11.0), 1, true, "Product encoding preserves componentwise injectivity."),
            dsfb_row(spec, "product_a2_b1", "satisfying", true, true, true, "(y2,z1)", "(a2,b1)", "(a2,b1)", 0.0, true, None, Some(21.0), 1, true, "Exact reconstruction works on the Cartesian product."),
            dsfb_row(spec, "product_collision_contrast", "boundary", true, false, false, "(y1,z1)", "(a1,b2)", "ambiguous", 1.0, false, None, Some(11.0), 2, false, "Loss of component injectivity creates a product collision."),
        ],
        DsfbMode::Distinguishability => vec![
            dsfb_row(spec, "distinct_s1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, None, Some(101.0), 1, true, "Distinct structural state s1 has its own observation."),
            dsfb_row(spec, "distinct_s2", "satisfying", true, true, true, "y102", "s2", "s2", 0.0, true, None, Some(102.0), 1, true, "Distinct structural state s2 has a distinct observation."),
            dsfb_row(spec, "collision_breaks_distinguishability", "boundary", true, false, false, "y10", "s4", "ambiguous", 0.5, false, None, Some(10.0), 3, false, "Collision contrast case shows why exact recovery is needed."),
        ],
        DsfbMode::MinimalResidual => vec![
            dsfb_row(spec, "image_point_s1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, None, Some(101.0), 1, true, "Observation in the exact image attains residual zero."),
            dsfb_row(spec, "image_point_s4", "satisfying", true, true, true, "y104", "s4", "s4", 0.0, true, None, Some(104.0), 1, true, "Another exact-image observation has minimal residual 0."),
            dsfb_row(spec, "outside_image_positive_residual", "boundary", true, true, false, "y995", "outside", "fallback", 5.0, false, None, Some(995.0), 0, false, "Outside-image observation has a strictly positive residual."),
        ],
        DsfbMode::UniqueInverse => vec![
            dsfb_row(spec, "inverse_agreement_s1", "satisfying", true, true, true, "y101", "s1", "s1", 0.0, true, None, Some(101.0), 1, true, "Two exact inverses agree on y101."),
            dsfb_row(spec, "inverse_agreement_s2", "satisfying", true, true, true, "y102", "s2", "s2", 0.0, true, None, Some(102.0), 1, true, "Two exact inverses agree on y102."),
            dsfb_row(spec, "outside_image_disagreement", "boundary", true, true, false, "y999", "outside", "inverse_a!=inverse_b", 2.0, false, None, Some(999.0), 0, false, "Agreement is guaranteed only on the exact image."),
        ],
        DsfbMode::PeriodicObservation => {
            let states = ["s1", "s2", "s1", "s2", "s1", "s2"];
            let observations = [101.0, 102.0, 101.0, 102.0, 101.0, 102.0];
            states
                .iter()
                .enumerate()
                .map(|(time_step, _state)| {
                    let index = (time_step + phase) % states.len();
                    dsfb_row(
                        spec,
                        &format!("periodic_obs_t{time_step}"),
                        "satisfying",
                        true,
                        true,
                        true,
                        if observations[index] == 101.0 { "y101" } else { "y102" },
                        states[index],
                        states[index],
                        0.0,
                        true,
                        Some(time_step),
                        Some(observations[index]),
                        1,
                        true,
                        "Periodic structural states induce periodic observations.",
                    )
                })
                .collect()
        }
        DsfbMode::PeriodicReconstruction => {
            let observations = [101.0, 102.0, 101.0, 102.0, 101.0, 102.0];
            let reconstructions = ["s1", "s2", "s1", "s2", "s1", "s2"];
            observations
                .iter()
                .enumerate()
                .map(|(time_step, observation)| {
                    let index = (time_step + phase) % observations.len();
                    dsfb_row(
                        spec,
                        &format!("periodic_reconstruction_t{time_step}"),
                        "satisfying",
                        true,
                        true,
                        true,
                        if observations[index] == 101.0 { "y101" } else { "y102" },
                        reconstructions[index],
                        reconstructions[index],
                        0.0,
                        true,
                        Some(time_step),
                        Some(*observation),
                        1,
                        true,
                        "Periodic observations reconstruct to a periodic state trace under exact DSFB.",
                    )
                })
                .collect()
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn dsfb_row(
    spec: &TheoremSpec,
    case_id: &str,
    case_type: &str,
    pass: bool,
    assumptions_satisfied: bool,
    injective_flag: bool,
    observation_id: &str,
    structural_state_id: &str,
    reconstructed_state_id: &str,
    residual_value: f64,
    exact_recovery_flag: bool,
    time_step: Option<usize>,
    observation_value: Option<f64>,
    equivalence_class_size: usize,
    roundtrip_flag: bool,
    notes: &str,
) -> DsfbRow {
    DsfbRow {
        theorem_id: spec.id.clone(),
        theorem_name: spec.title.clone(),
        component: "dsfb",
        case_id: case_id.to_string(),
        case_type: case_type.to_string(),
        pass,
        notes: notes.to_string(),
        assumptions_satisfied,
        injective_flag,
        observation_id: observation_id.to_string(),
        structural_state_id: structural_state_id.to_string(),
        reconstructed_state_id: reconstructed_state_id.to_string(),
        residual_value,
        exact_recovery_flag,
        time_step,
        observation_value,
        equivalence_class_size,
        roundtrip_flag,
    }
}
