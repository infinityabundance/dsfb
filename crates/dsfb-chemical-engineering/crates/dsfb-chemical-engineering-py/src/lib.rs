//! Python bindings (pyo3) for a focused, pure subset of the DSFB-Chemical-Engineering edge crate.
//!
//! Exposes deterministic, file-free helpers — version, the unit-consistency hazard classifier, and the
//! industrial-data-readiness grader — so a Python user can call the read-only courts without leaving Python.
//! The heavyweight pipeline (`analyze`, `casefile`) stays in the Rust binary / container; these bindings are
//! the thin, marshalling-cheap surface. The wheel is built + published by the user with maturin (USER-ONLY).
//!
//! Bounded: same non-claims as the underlying objects — advisory, read-only, no control/safety authority.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use dsfb_chemical_engineering_edge as edge;

/// Crate version string.
#[pyfunction]
fn version() -> &'static str {
    edge::VERSION
}

/// Classify a pair of engineering-unit strings into a unit-consistency hazard tag
/// (`consistent` / `scale_mismatch` / `affine_offset_hazard` / `dimension_mismatch` / `basis_mismatch` /
/// `unknown_unit`) — the deterministic unit court applied to one `same-quantity` assertion.
#[pyfunction]
fn classify_unit_pair(a: &str, b: &str) -> String {
    use edge::unit_consistency::{UnitAssertion, UnitConsistencyCourtV1, UnitRelation};
    let assertion = UnitAssertion {
        context: "py".into(),
        a_channel: "a".into(),
        a_unit: a.into(),
        b_channel: "b".into(),
        b_unit: b.into(),
        relation: UnitRelation::SameQuantity,
    };
    let court = UnitConsistencyCourtV1::build(&[assertion]);
    court.findings[0].hazard.clone()
}

/// Grade a historian-export profile (the deterministic `IndustrialDataReadinessCourtV1`). Returns a dict with
/// `verdict`, `n_caveat`, `n_critical_missing`, and the sealed `court_hash`. Optional context fields default
/// to absent (a caveat), matching the Rust default profile.
#[pyfunction]
#[pyo3(signature = (n_tags, n_rows, baseline_present, license_cleared, missingness, unit_coverage,
    duplicate_timestamps=0.0, time_span_hours=f64::NAN, sampling_regular=true,
    has_controllers=false, has_maintenance=false, has_batches=false, has_lab_samples=false))]
#[allow(clippy::too_many_arguments)]
fn grade_readiness<'py>(
    py: Python<'py>,
    n_tags: usize,
    n_rows: usize,
    baseline_present: bool,
    license_cleared: bool,
    missingness: f64,
    unit_coverage: f64,
    duplicate_timestamps: f64,
    time_span_hours: f64,
    sampling_regular: bool,
    has_controllers: bool,
    has_maintenance: bool,
    has_batches: bool,
    has_lab_samples: bool,
) -> PyResult<Bound<'py, PyDict>> {
    use edge::data_readiness::{HistorianExportProfile, IndustrialDataReadinessCourtV1};
    let profile = HistorianExportProfile {
        n_tags,
        n_rows,
        baseline_window_present: baseline_present,
        license_cleared,
        missingness_fraction: missingness,
        unit_coverage_fraction: unit_coverage,
        duplicate_timestamp_fraction: duplicate_timestamps,
        time_span_hours,
        sampling_regular,
        has_controllers,
        has_maintenance,
        has_batches,
        has_lab_samples,
    };
    let court = IndustrialDataReadinessCourtV1::grade(&profile);
    let d = PyDict::new(py);
    d.set_item("verdict", court.verdict)?;
    d.set_item("n_caveat", court.n_caveat)?;
    d.set_item("n_critical_missing", court.n_critical_missing)?;
    d.set_item("court_hash", court.court_hash)?;
    Ok(d)
}

/// The Python module `dsfb_chemical_engineering`.
#[pymodule]
fn dsfb_chemical_engineering(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(classify_unit_pair, m)?)?;
    m.add_function(wrap_pyfunction!(grade_readiness, m)?)?;
    Ok(())
}
