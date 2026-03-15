use serde::Serialize;

use crate::registry::TheoremSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CaseClass {
    Passing,
    Boundary,
    Violating,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaseMetadata {
    pub theorem_id: String,
    pub theorem_name: String,
    pub component: &'static str,
    pub case_id: String,
    pub case_class: CaseClass,
    pub assumption_satisfied: bool,
    pub expected_outcome: String,
    pub observed_outcome: String,
    pub pass: bool,
    pub notes: String,
}

impl CaseMetadata {
    pub fn new(
        spec: &TheoremSpec,
        component: &'static str,
        case_id: impl Into<String>,
        case_class: CaseClass,
        assumption_satisfied: bool,
        expected_outcome: impl Into<String>,
        observed_outcome: impl Into<String>,
        pass: bool,
        notes: impl Into<String>,
    ) -> Self {
        Self {
            theorem_id: spec.id.clone(),
            theorem_name: spec.title.clone(),
            component,
            case_id: case_id.into(),
            case_class,
            assumption_satisfied,
            expected_outcome: expected_outcome.into(),
            observed_outcome: observed_outcome.into(),
            pass,
            notes: notes.into(),
        }
    }

    pub fn pass(&self) -> bool {
        self.pass
    }
}
