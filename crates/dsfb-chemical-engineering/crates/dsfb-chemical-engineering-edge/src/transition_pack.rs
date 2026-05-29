//! `SBIRTransitionPackV1` (P69) — a machine + human Phase-I-style transition / readiness pack.
//!
//! A Phase-I-style effort closes by handing a reviewer a *transition pack*: what was attempted, which
//! go/no-go gates were met, what is ready vs not, the residual risks, and how to reproduce the result.
//! This object packages exactly that, **deterministically and hash-sealed**, reusing the milestone-gated
//! evaluation protocol introduced in P41 (M0–M3 go/no-go gates, each anchored to a replay-checkable
//! artifact). It is simultaneously:
//! - **machine-readable** — a sealed, serde-serialisable record with a `pack_hash` and `verify()`; and
//! - **human-readable** — [`SBIRTransitionPackV1::to_markdown`] renders a one-page pack a reviewer reads.
//!
//! Bounded honestly: it is **generic — it names no agency, program, or vendor**, every readiness claim
//! carries an explicit `boundary_note` of what is *not* claimed, every milestone is anchored to a sealed
//! evidence hash, and a `non_claims` block states the limits up front. It asserts no root cause, no
//! causality, and no operational fitness beyond what the anchored artifacts demonstrate. Additive + off
//! the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Status of a milestone gate. `OutOfScope` is an honest "not attempted in this phase" — distinct from
/// `Pending` (attempted, gate not yet met).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    Met,
    Pending,
    OutOfScope,
}

impl MilestoneStatus {
    /// Stable tag fed to the hasher and rendered in the pack (never localise/reorder — part of the seal).
    fn tag(self) -> &'static str {
        match self {
            MilestoneStatus::Met => "met",
            MilestoneStatus::Pending => "pending",
            MilestoneStatus::OutOfScope => "out_of_scope",
        }
    }
}

/// Residual-risk level after the stated mitigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    fn tag(self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
        }
    }
}

/// One milestone gate (mirrors the P41 M0–M3 protocol): an objective with a replay-checkable go/no-go
/// gate, anchored to the hash of the artifact that satisfies it, plus its status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionMilestone {
    /// Milestone id (e.g. `"M0"`, `"M1"`).
    pub id: String,
    pub objective: String,
    /// The replay-checkable go/no-go condition (prose, e.g. "verify-replay 6/6 byte-identical").
    pub go_no_go_gate: String,
    /// SHA-256 of the sealed artifact that demonstrates this gate (or `""` when out of scope).
    pub evidence_anchor: String,
    pub status: MilestoneStatus,
}

/// One readiness claim: a capability, whether it was demonstrated, the anchoring evidence, and the
/// explicit boundary of what is *not* being claimed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessClaim {
    pub capability: String,
    pub demonstrated: bool,
    pub evidence_anchor: String,
    /// What this claim deliberately does **not** assert (bounded honesty).
    pub boundary_note: String,
}

/// One risk-register entry: the risk, its mitigation, and the residual level after mitigation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskItem {
    pub risk: String,
    pub mitigation: String,
    pub residual: RiskLevel,
}

/// A hash-sealed, self-verifying Phase-I-style transition / readiness pack (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SBIRTransitionPackV1 {
    /// A generic project label (no agency/program/vendor name).
    pub project_label: String,
    pub problem_statement: String,
    pub milestones: Vec<TransitionMilestone>,
    pub readiness_claims: Vec<ReadinessClaim>,
    pub risks: Vec<RiskItem>,
    /// Ordered, copy-pasteable reproduction steps (commands / verifiers).
    pub reproduction_steps: Vec<String>,
    /// Explicit limits stated up front (e.g. "asserts no root cause").
    pub non_claims: Vec<String>,
    pub pack_hash: String,
}

impl SBIRTransitionPackV1 {
    #[allow(clippy::too_many_arguments)]
    fn seal(
        project_label: &str,
        problem_statement: &str,
        milestones: &[TransitionMilestone],
        readiness_claims: &[ReadinessClaim],
        risks: &[RiskItem],
        reproduction_steps: &[String],
        non_claims: &[String],
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"sbir_transition_pack_v1");
        h.field("project_label", project_label.as_bytes());
        h.field("problem_statement", problem_statement.as_bytes());
        for m in milestones {
            h.field("milestone_id", m.id.as_bytes());
            h.field("objective", m.objective.as_bytes());
            h.field("go_no_go_gate", m.go_no_go_gate.as_bytes());
            h.field("evidence_anchor", m.evidence_anchor.as_bytes());
            h.field("status", m.status.tag().as_bytes());
        }
        for c in readiness_claims {
            h.field("capability", c.capability.as_bytes());
            h.u64("demonstrated", c.demonstrated as u64);
            h.field("claim_anchor", c.evidence_anchor.as_bytes());
            h.field("boundary_note", c.boundary_note.as_bytes());
        }
        for r in risks {
            h.field("risk", r.risk.as_bytes());
            h.field("mitigation", r.mitigation.as_bytes());
            h.field("residual", r.residual.tag().as_bytes());
        }
        for s in reproduction_steps {
            h.field("reproduction_step", s.as_bytes());
        }
        for n in non_claims {
            h.field("non_claim", n.as_bytes());
        }
        h.finalize_hex()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build(
        project_label: impl Into<String>,
        problem_statement: impl Into<String>,
        milestones: Vec<TransitionMilestone>,
        readiness_claims: Vec<ReadinessClaim>,
        risks: Vec<RiskItem>,
        reproduction_steps: Vec<String>,
        non_claims: Vec<String>,
    ) -> Self {
        let project_label = project_label.into();
        let problem_statement = problem_statement.into();
        let pack_hash = Self::seal(
            &project_label,
            &problem_statement,
            &milestones,
            &readiness_claims,
            &risks,
            &reproduction_steps,
            &non_claims,
        );
        SBIRTransitionPackV1 {
            project_label,
            problem_statement,
            milestones,
            readiness_claims,
            risks,
            reproduction_steps,
            non_claims,
            pack_hash,
        }
    }

    /// Re-derive the seal from the record's own fields.
    pub fn verify(&self) -> bool {
        Self::seal(
            &self.project_label,
            &self.problem_statement,
            &self.milestones,
            &self.readiness_claims,
            &self.risks,
            &self.reproduction_steps,
            &self.non_claims,
        ) == self.pack_hash
    }

    /// Number of milestones whose gate is `Met`.
    pub fn n_met(&self) -> usize {
        self.milestones
            .iter()
            .filter(|m| m.status == MilestoneStatus::Met)
            .count()
    }

    /// Number of milestones in scope (anything not `OutOfScope`).
    pub fn n_in_scope(&self) -> usize {
        self.milestones
            .iter()
            .filter(|m| m.status != MilestoneStatus::OutOfScope)
            .count()
    }

    /// First 8 hex chars of an anchor, for compact human rendering (`—` when empty).
    fn anchor_short(anchor: &str) -> &str {
        if anchor.is_empty() {
            "—"
        } else {
            &anchor[..8.min(anchor.len())]
        }
    }

    /// Render the pack as a one-page Markdown document for a human reviewer.
    pub fn to_markdown(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "# Transition readiness pack — {}\n\n",
            self.project_label
        ));
        s.push_str(&format!("**Problem.** {}\n\n", self.problem_statement));

        s.push_str("## Milestone gates (go/no-go)\n\n");
        s.push_str(&format!(
            "Gates met: **{}/{}** in scope.\n\n",
            self.n_met(),
            self.n_in_scope()
        ));
        s.push_str("| ID | Objective | Go/no-go gate | Status | Evidence |\n");
        s.push_str("|---|---|---|---|---|\n");
        for m in &self.milestones {
            s.push_str(&format!(
                "| {} | {} | {} | {} | `{}…` |\n",
                m.id,
                m.objective,
                m.go_no_go_gate,
                m.status.tag(),
                Self::anchor_short(&m.evidence_anchor),
            ));
        }

        s.push_str("\n## Readiness claims\n\n");
        for c in &self.readiness_claims {
            let mark = if c.demonstrated {
                "demonstrated"
            } else {
                "not demonstrated"
            };
            s.push_str(&format!(
                "- **{}** — {} (`{}…`). Boundary: {}\n",
                c.capability,
                mark,
                Self::anchor_short(&c.evidence_anchor),
                c.boundary_note,
            ));
        }

        s.push_str("\n## Risk register\n\n");
        s.push_str("| Risk | Mitigation | Residual |\n|---|---|---|\n");
        for r in &self.risks {
            s.push_str(&format!(
                "| {} | {} | {} |\n",
                r.risk,
                r.mitigation,
                r.residual.tag()
            ));
        }

        s.push_str("\n## Reproduction\n\n");
        for (i, step) in self.reproduction_steps.iter().enumerate() {
            s.push_str(&format!("{}. {}\n", i + 1, step));
        }

        s.push_str("\n## Non-claims (bounded honestly)\n\n");
        for n in &self.non_claims {
            s.push_str(&format!("- {n}\n"));
        }

        s.push_str(&format!("\n---\nPack hash: `{}`\n", self.pack_hash));
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pack() -> SBIRTransitionPackV1 {
        // Milestone gates mirroring the P41 M0–M3 protocol, anchored to sealed artifacts.
        let milestones = vec![
            TransitionMilestone {
                id: "M0".into(),
                objective: "Deterministic replay of the residual-semiotics pipeline".into(),
                go_no_go_gate: "verify-replay 6/6 byte-identical".into(),
                evidence_anchor: "a".repeat(64),
                status: MilestoneStatus::Met,
            },
            TransitionMilestone {
                id: "M1".into(),
                objective: "Balance witness catches a spoofed-sensor incident".into(),
                go_no_go_gate: "closure residual crosses threshold on post-onset window".into(),
                evidence_anchor: "b".repeat(64),
                status: MilestoneStatus::Met,
            },
            TransitionMilestone {
                id: "M2".into(),
                objective: "GPU evidence sealing byte-identical to CPU reference".into(),
                go_no_go_gate: "digest-equivalence harness passes; evidence_root unchanged".into(),
                evidence_anchor: "c".repeat(64),
                status: MilestoneStatus::Met,
            },
            TransitionMilestone {
                id: "M3".into(),
                objective: "On-target MCU deployment".into(),
                go_no_go_gate: "fixed-point core runs within bounded memory on Cortex-M".into(),
                evidence_anchor: String::new(),
                status: MilestoneStatus::OutOfScope, // design only this phase (P55)
            },
        ];
        let claims = vec![ReadinessClaim {
            capability: "Replayable court record over established-detector residuals".into(),
            demonstrated: true,
            evidence_anchor: "d".repeat(64),
            boundary_note: "advisory structural episodes only; asserts no root cause".into(),
        }];
        let risks = vec![RiskItem {
            risk: "Real multi-plant fleet data unavailable for validation".into(),
            mitigation: "Synthetic multi-unit demonstrator with declared residence times".into(),
            residual: RiskLevel::Medium,
        }];
        let repro = vec![
            "cargo test --workspace".into(),
            "cargo run -p dsfb-chemical-engineering-edge --bin dsfb-chem-edge -- verify-replay"
                .into(),
        ];
        let non_claims = vec![
            "Asserts no root cause and no causality.".into(),
            "Names no agency, program, or vendor.".into(),
            "Replaces no estimator, controller, historian, or alarm system.".into(),
        ];
        SBIRTransitionPackV1::build("Residual-semiotics fault-monitoring layer", "Established chemometric detectors emit residuals but no auditable, replayable structural record.", milestones, claims, risks, repro, non_claims)
    }

    #[test]
    fn pack_seals_and_self_verifies() {
        let pack = sample_pack();
        assert!(pack.verify());
        assert_eq!(pack.n_met(), 3);
        assert_eq!(pack.n_in_scope(), 3); // M3 is out of scope
        assert_eq!(pack.milestones.len(), 4);
    }

    #[test]
    fn tampering_any_field_breaks_the_seal() {
        let mut pack = sample_pack();
        assert!(pack.verify());
        // Flip an out-of-scope gate to "met" without re-sealing → verify must fail.
        pack.milestones[3].status = MilestoneStatus::Met;
        assert!(!pack.verify());
    }

    #[test]
    fn markdown_renders_all_sections_and_is_agency_free() {
        let md = sample_pack().to_markdown();
        for section in [
            "Milestone gates",
            "Readiness claims",
            "Risk register",
            "Reproduction",
            "Non-claims",
        ] {
            assert!(md.contains(section), "missing section: {section}");
        }
        assert!(md.contains("Gates met: **3/3**"));
        // Generic: the pack must not name a specific agency/program.
        for banned in ["DARPA", "AFWERX", "DoD", "NASA", "NSF"] {
            assert!(!md.contains(banned), "leaked agency name: {banned}");
        }
    }
}
