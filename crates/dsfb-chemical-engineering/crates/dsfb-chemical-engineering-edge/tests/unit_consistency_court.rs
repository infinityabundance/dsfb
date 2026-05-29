//! Integration test for the `UnitConsistencyCourtV1` deriver (Wave-3 physics) against the **real**
//! shipped instrumented balances.
//!
//! The module's own unit tests prove the court bites (it flags °C-vs-K, bar-vs-Pa, wt%-vs-mol%). This
//! test pins the other half of the contract: when the deriver is run over DSFB's *own* documented
//! balances, every balance is unit-consistent and every verdict self-verifies — so the demonstrator's
//! clean result is a tested property, not a one-off CLI run. If anyone edits a roles sidecar to combine
//! mismatched units (a level in `cm` against one in `m`, a flow in `kg/h` summed with `kg/s`), this test
//! fails, exactly as the court is meant to catch.

use std::fs;

use dsfb_chemical_engineering_edge::balance::RolesDoc;
use dsfb_chemical_engineering_edge::unit_consistency::{
    assertions_for_balance, UnitConsistencyCourtV1,
};

#[test]
fn every_documented_balance_is_unit_consistent() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("instrumented");
    let mut roles_files: Vec<_> = fs::read_dir(&dir)
        .expect("instrumented dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().ends_with(".roles.json"))
        .collect();
    roles_files.sort();
    assert!(
        !roles_files.is_empty(),
        "expected instrumented roles sidecars in {}",
        dir.display()
    );

    let mut checked = 0usize;
    for rf in &roles_files {
        let roles = RolesDoc::load(rf).unwrap_or_else(|e| panic!("load {}: {e}", rf.display()));
        let asserts = assertions_for_balance(&roles);
        if asserts.is_empty() {
            continue; // a balance type with no additively-combined channel group to check
        }
        let court = UnitConsistencyCourtV1::build(&asserts);
        assert!(court.verify(), "{}: court must self-verify", roles.dataset);
        assert!(
            court.all_consistent(),
            "{}: documented balance has a unit hazard — {}",
            roles.dataset,
            court.render()
        );
        checked += 1;
    }
    // All six instrumented demonstrators declare a balance whose combined channels we can check.
    assert!(
        checked >= 6,
        "expected ≥6 balances with checkable unit groups, checked {checked}"
    );
}
