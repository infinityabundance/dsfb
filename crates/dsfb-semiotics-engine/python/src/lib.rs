use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{EngineConfig, StructuralSemioticsEngine};
use dsfb_semiotics_engine::engine::types::EnvelopeMode;
use dsfb_semiotics_engine::live::{to_real, LiveEngineStatus, OnlineStructuralEngine};
use dsfb_semiotics_engine::math::envelope::EnvelopeSpec;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

fn py_runtime_error(error: impl ToString) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

// TRACE:INTERFACE:IFACE-PYTHON-BINDING:Python bindings over deterministic engine:Exposes bounded live status and deterministic batch summaries to Python and Jupyter users.
fn status_to_dict<'py>(
    py: Python<'py>,
    status: &dsfb_semiotics_engine::live::LiveEngineStatus,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new_bound(py);
    dict.set_item("step", status.step)?;
    dict.set_item("time", status.time)?;
    dict.set_item("syntax_label", status.syntax_label.clone())?;
    dict.set_item("grammar_state", format!("{:?}", status.grammar_state))?;
    dict.set_item("grammar_reason_text", status.grammar_reason_text.clone())?;
    dict.set_item("semantic_disposition", status.semantic_disposition.clone())?;
    dict.set_item("trust_scalar", status.trust_scalar)?;
    dict.set_item("selected_heuristic_ids", status.selected_heuristic_ids.clone())?;
    Ok(dict)
}

fn bundle_summary_to_dict<'py>(
    py: Python<'py>,
    bundle: dsfb_semiotics_engine::EngineOutputBundle,
) -> PyResult<Bound<'py, PyDict>> {
    let scenario = bundle
        .scenario_outputs
        .first()
        .ok_or_else(|| py_runtime_error("bundle contained no scenario outputs"))?;
    let summary = PyDict::new_bound(py);
    summary.set_item("scenario_id", scenario.record.id.clone())?;
    summary.set_item("input_mode", bundle.run_metadata.input_mode)?;
    summary.set_item("syntax_label", scenario.syntax.trajectory_label.clone())?;
    summary.set_item(
        "grammar_reason_text",
        scenario
            .grammar
            .last()
            .map(|status| status.reason_text.clone())
            .unwrap_or_default(),
    )?;
    summary.set_item(
        "semantic_disposition",
        format!("{:?}", scenario.semantics.disposition),
    )?;
    summary.set_item(
        "selected_heuristics",
        scenario.semantics.selected_heuristic_ids.clone(),
    )?;
    summary.set_item(
        "trust_scalar",
        scenario
            .grammar
            .last()
            .map(|status| status.trust_scalar.value())
            .unwrap_or(1.0),
    )?;
    Ok(summary)
}

#[pyclass(name = "SemioticsEngine")]
struct PySemioticsEngine {
    inner: OnlineStructuralEngine,
    latest_status: Option<LiveEngineStatus>,
}

#[pymethods]
impl PySemioticsEngine {
    #[new]
    #[pyo3(signature = (history_buffer_capacity=64, envelope_radius=1.0, dt=1.0))]
    fn new(history_buffer_capacity: usize, envelope_radius: f64, dt: f64) -> PyResult<Self> {
        let mut settings = dsfb_semiotics_engine::EngineSettings::default();
        settings.online.history_buffer_capacity = history_buffer_capacity;
        let inner = OnlineStructuralEngine::with_builtin_bank(
            "python_live_engine",
            vec!["residual".to_string()],
            dt,
            EnvelopeSpec {
                name: "python_fixed_envelope".to_string(),
                mode: EnvelopeMode::Fixed,
                base_radius: envelope_radius,
                slope: 0.0,
                switch_step: None,
                secondary_slope: None,
                secondary_base: None,
            },
            settings,
        )
        .map_err(py_runtime_error)?;
        Ok(Self {
            inner,
            latest_status: None,
        })
    }

    fn push_sample(&mut self, py: Python<'_>, time: f64, residual_value: f64) -> PyResult<PyObject> {
        let status = self
            .inner
            .push_residual_sample(time, &[to_real(residual_value)])
            .map_err(py_runtime_error)?;
        self.latest_status = Some(status.clone());
        Ok(status_to_dict(py, &status)?.unbind().into())
    }

    fn current_status(&self, py: Python<'_>) -> PyResult<PyObject> {
        let status = self
            .latest_status
            .as_ref()
            .ok_or_else(|| py_runtime_error("no samples have been pushed yet"))?;
        Ok(status_to_dict(py, status)?.unbind().into())
    }
}

#[pyfunction]
fn run_scenario(py: Python<'_>, scenario_id: &str) -> PyResult<PyObject> {
    let bundle = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        scenario_id,
    ))
    .run_selected()
    .map_err(py_runtime_error)?;
    Ok(bundle_summary_to_dict(py, bundle)?.unbind().into())
}

#[pyfunction]
#[pyo3(signature = (observed_csv, predicted_csv, scenario_id="python_csv_case", time_column="time", dt=1.0))]
fn run_csv(
    py: Python<'_>,
    observed_csv: &str,
    predicted_csv: &str,
    scenario_id: &str,
    time_column: &str,
    dt: f64,
) -> PyResult<PyObject> {
    let bundle = StructuralSemioticsEngine::new(EngineConfig::csv(
        CommonRunConfig {
            dt,
            ..Default::default()
        },
        CsvInputConfig {
            observed_csv: observed_csv.into(),
            predicted_csv: predicted_csv.into(),
            scenario_id: scenario_id.to_string(),
            channel_names: None,
            time_column: Some(time_column.to_string()),
            dt_fallback: dt,
            envelope_mode: EnvelopeMode::Fixed,
            envelope_base: 1.0,
            envelope_slope: 0.0,
            envelope_switch_step: None,
            envelope_secondary_slope: None,
            envelope_secondary_base: None,
            envelope_name: "python_csv_envelope".to_string(),
        },
    ))
    .run_selected()
    .map_err(py_runtime_error)?;
    Ok(bundle_summary_to_dict(py, bundle)?.unbind().into())
}

#[pyfunction]
fn run_array(py: Python<'_>, values: Vec<f64>) -> PyResult<PyObject> {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "python_array",
        vec!["residual".to_string()],
        1.0,
        EnvelopeSpec {
            name: "python_array_envelope".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        dsfb_semiotics_engine::EngineSettings::default(),
    )
    .map_err(py_runtime_error)?;

    let items = PyList::empty_bound(py);
    for (index, value) in values.into_iter().enumerate() {
        let status = engine
            .push_residual_sample(index as f64, &[to_real(value)])
            .map_err(py_runtime_error)?;
        items.append(status_to_dict(py, &status)?)?;
    }
    Ok(items.unbind().into())
}

#[pymodule]
fn dsfb_engine(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PySemioticsEngine>()?;
    module.add_function(wrap_pyfunction!(run_scenario, module)?)?;
    module.add_function(wrap_pyfunction!(run_csv, module)?)?;
    module.add_function(wrap_pyfunction!(run_array, module)?)?;
    Ok(())
}
