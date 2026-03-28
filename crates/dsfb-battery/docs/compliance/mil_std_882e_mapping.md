# MIL-STD-882E Mapping

Status: Partial

This file provides a mishap-mitigation mapping scaffold. It is not a completed system safety assessment.

Relevant crate components:
- `src/types.rs::ReasonCode`
- `src/types.rs::GrammarState`
- `src/detection.rs::assign_reason_code`
- `src/compliance.rs` operator overlay

Traceability matrix:

| DSFB signal | Candidate hazard linkage | Advisory mitigation action | Crate component | Status |
|---|---|---|---|---|
| `Boundary` | Emerging structural deviation | Review trend, persistence, and host-side monitoring policy | `evaluate_grammar_state`, operator overlay | Partial |
| `Violation` | Envelope exit / elevated degradation risk | Escalate host review; consider protection policy in the external system | `evaluate_grammar_state`, operator overlay | Partial |
| `SustainedCapacityFade` | Endurance margin erosion | Maintenance planning, reserve review, repeated monitoring | `assign_reason_code` | Partial |
| `AcceleratingFadeKnee` | Rapid loss-of-margin onset candidate | Escalated inspection under host policy | `assign_reason_code`, `build_knee_onset_narrative` | Partial |

The mapping is advisory only. The crate does not perform mishap severity or probability approval activities.
