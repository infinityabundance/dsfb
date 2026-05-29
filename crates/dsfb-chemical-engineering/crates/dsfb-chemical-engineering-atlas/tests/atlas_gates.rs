//! Authority-gate and hash-determinism tests for the atlas crate.

use atlas::detector::{ImplementationStatus, DETECTOR_RECORDS};
use atlas::heuristics::HEURISTIC_RECORDS;
use dsfb_chemical_engineering_atlas as atlas;

#[test]
fn validate_admits_the_bundled_atlas() {
    let r = atlas::validate().expect("bundled atlas validates");
    assert_eq!(r.n_detectors, 18);
    assert_eq!(r.n_executed, 18);
    assert_eq!(r.n_heuristics, 6);
    assert!(r.n_primitive_families >= 6, "broad primitive-family spread");
    assert_eq!(r.n_fault_signatures, 12);
    // Seven signatures are executed by the project's witnesses/demonstrators: F6 leak + F7 sensor-drift
    // (balance/isolation witnesses) and F1 stiction, F2 pump cavitation, F3 heat-transfer fouling,
    // F8 controller masking, F9 valve-hunting (synthetic demonstrators gated by edge
    // `tests/fault_demonstrators.rs`). The rest remain honestly catalogued.
    assert_eq!(
        r.n_fault_signatures_executed, 7,
        "F1, F2, F3, F6, F7, F8, F9 are executed"
    );
}

#[test]
fn fault_signatures_unique_sourced_and_reference_real_detectors() {
    use atlas::fault_signature::FAULT_SIGNATURES;
    let det_ids: Vec<&str> = DETECTOR_RECORDS.iter().map(|d| d.detector_id).collect();
    for (i, f) in FAULT_SIGNATURES.iter().enumerate() {
        assert!(
            !f.source_refs.is_empty() && !f.exhibiting_datasets.is_empty(),
            "{} unsourced",
            f.fault_id
        );
        for inp in f.detector_inputs {
            // detector inputs are either a catalogued detector id or the named balance witness.
            assert!(
                det_ids.contains(inp) || *inp == "mass_energy_balance_witness",
                "fault {} input '{}' is not a catalogued detector",
                f.fault_id,
                inp
            );
        }
        for b in &FAULT_SIGNATURES[i + 1..] {
            assert_ne!(f.fault_id, b.fault_id, "duplicate fault_id");
        }
    }
}

#[test]
fn detector_ids_and_canonical_ids_are_unique() {
    for (i, a) in DETECTOR_RECORDS.iter().enumerate() {
        for b in &DETECTOR_RECORDS[i + 1..] {
            assert_ne!(a.detector_id, b.detector_id, "duplicate detector_id");
            assert_ne!(a.canonical_id, b.canonical_id, "duplicate canonical_id");
        }
    }
}

#[test]
fn every_detector_has_a_source_ref_and_is_executed() {
    for d in DETECTOR_RECORDS {
        assert!(
            !d.source_refs.is_empty(),
            "{} missing source_refs",
            d.detector_id
        );
        assert_eq!(d.implementation_status, ImplementationStatus::Executed);
    }
}

#[test]
fn h1_through_h6_present_with_inputs_and_modes() {
    for id in ["H1", "H2", "H3", "H4", "H5", "H6"] {
        let n = HEURISTIC_RECORDS
            .iter()
            .filter(|h| h.heuristic_id == id)
            .count();
        assert_eq!(n, 1, "{id} must appear exactly once");
    }
    for h in HEURISTIC_RECORDS {
        assert!(!h.detector_inputs.is_empty());
        assert!(!h.known_false_positive_modes.is_empty());
        assert!(!h.known_false_negative_modes.is_empty());
        assert!(!h.admissibility_condition.is_empty());
    }
}

#[test]
fn hashes_are_deterministic_and_64_hex() {
    for (a, b) in [
        (
            atlas::hashes::detector_atlas_hash_v1(),
            atlas::hashes::detector_atlas_hash_v1(),
        ),
        (
            atlas::hashes::chemical_process_heuristics_hash_v1(),
            atlas::hashes::chemical_process_heuristics_hash_v1(),
        ),
        (
            atlas::hashes::challenge_docket_hash_v1(),
            atlas::hashes::challenge_docket_hash_v1(),
        ),
        (
            atlas::hashes::atlas_hash_v1(),
            atlas::hashes::atlas_hash_v1(),
        ),
    ] {
        assert_eq!(a, b, "hash must be deterministic across calls");
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn atlas_hash_v1_is_frozen() {
    // Pins the composite digest of the bundled authority data. If a record changes intentionally,
    // recompute and update this expected value (that is the point of the gate).
    assert_eq!(atlas::hashes::atlas_hash_v1(), ATLAS_HASH_V1_EXPECTED);
}

// Re-frozen in P40 (F1+F9 promoted), P51 (F3+F8 promoted Catalogued -> Executed), and P60 (the 7-class
// UnknownTaxonomyV1 joined the preimage via `unknown_taxonomy_hash_v1`). Each is an intentional
// authority-record change; the cuda Passport `atlas_hash` value tracks it but the CUDA `evidence_root`
// and the edge court `bundle_root` are unaffected (the passport is excluded from the root, and the edge
// court record does not embed `atlas_hash`).
//
// GOVERNED RE-FREEZE (Phase-C executable, 2026-05-26): four prior-art detectors promoted Catalogued →
// Executed (mewma, dpca, mosum, mmd — the executed bank 14 → 18; the 57-detector census total is unchanged,
// catalogued 43 → 39), each with a matching edge implementation in the `build_bank`. The detector authority
// records changed, so `detector_atlas_hash_v1` and the composite `atlas_hash_v1` are re-minted.
//
// GOVERNED RE-FREEZE (Phase-C +1 signature, 2026-05-26): the F2 pump-cavitation fault signature is
// promoted Catalogued → Executed (executed signatures 6 → 7; the 12-signature bank total is unchanged),
// backed by the synthetic `cavitation_instrumented` demonstrator gated in edge
// `tests/fault_demonstrators.rs` (broadband-vibration SPE motif → spectral_entropy_spe / knn_spe /
// ewma_spe). `fault_signatures_hash_v1` hashes `implementation_status`, so the composite `atlas_hash_v1`
// is re-minted below. (The cuda Passport `atlas_hash` tracks it; the CUDA `evidence_root` and the edge
// court `bundle_root` are unaffected — neither embeds `atlas_hash`.)
const ATLAS_HASH_V1_EXPECTED: &str =
    "936ac67a4e788685bcddae797418ff06e98a5f64a885f4cdfce2780002804ee3";
