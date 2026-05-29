//! `ChemicalAuthoritySeparationLawV1` — the executable doctrine of the five execution-vs-authority
//! separation rules (P57).
//!
//! DSFB-Chemical-Engineering separates **execution** (the `edge`/`cuda` crates: pipelines, kernels,
//! court records) from **authority** (the `atlas`/`corpus` crates: what evidence is *allowed to mean*).
//! Several existing tests enforce slices of that separation (the `atlas_authority_gate` subset test,
//! the both-backends hash equality in the cuda CLI). This module names the doctrine **as a whole** —
//! the five rules — and *executably* re-checks the runtime-checkable ones, so a drift (an executed
//! detector that isn't catalogued, an authority hash that isn't deterministic) fails a single gate.
//!
//! Two of the five rules are **compile-time structural** (the crate dependency graph + `#![no_std]` +
//! `const` records): they cannot fail at runtime if the workspace compiled at all, so the doctrine
//! reports them as enforced-by-construction with the mechanism named, rather than pretending to
//! re-verify them at runtime. The other three are checked here against the live records.

use crate::data::DataMatrix;
use crate::detectors::build_bank;
use crate::heuristics::canonical_bank;
use dsfb_chemical_engineering_atlas as authority;

/// One separation rule: its statement and how it is enforced.
#[derive(Debug, Clone, Copy)]
pub struct SeparationRule {
    /// Stable short id (e.g. `"R3_subset"`).
    pub id: &'static str,
    /// The doctrine statement.
    pub statement: &'static str,
    /// How the rule is enforced (a named gate, or the compile-time mechanism).
    pub enforcement: &'static str,
    /// `true` if this module re-checks the rule at runtime; `false` if it is compile-time structural.
    pub runtime_checkable: bool,
}

/// The five rules, in order. This `&'static` list is the doctrine of record.
pub const SEPARATION_RULES: &[SeparationRule] = &[
    SeparationRule {
        id: "R1_authority_is_pure",
        statement:
            "Authority crates (atlas, corpus) are `no_std` and depend on nothing — no execution \
                    types (FusedEpisode, GrammarState, kernels) ever leak into authority.",
        enforcement:
            "compile-time: atlas/corpus declare `#![cfg_attr(not(std), no_std)]` and carry no \
                      path-dependency on edge/cuda (see their Cargo.toml).",
        runtime_checkable: false,
    },
    SeparationRule {
        id: "R2_execution_depends_on_authority",
        statement:
            "Execution (edge, cuda) depends on authority, never the reverse — the arrow points \
                    one way.",
        enforcement:
            "compile-time: edge path-depends on atlas (+ optional corpus); authority has no \
                      back-edge. A cycle would not compile.",
        runtime_checkable: false,
    },
    SeparationRule {
        id: "R3_executed_is_subset_of_catalogued",
        statement: "Every detector the pipeline *executes* and every runtime heuristic (and its \
                    detector inputs) is catalogued in the atlas authority.",
        enforcement: "runtime: checked here (and by tests/atlas_authority_gate.rs).",
        runtime_checkable: true,
    },
    SeparationRule {
        id: "R4_one_authority_hash",
        statement:
            "The authority hash is single-sourced and deterministic — both execution backends \
                    print the identical frozen atlas_hash_v1 / corpus_hash_v1.",
        enforcement:
            "runtime: determinism checked here; cross-backend equality additionally gated by \
                      the cuda CLI/atlas tests (both call the same authority crate).",
        runtime_checkable: true,
    },
    SeparationRule {
        id: "R5_static_const_records",
        statement:
            "Authority records are `&'static const` tables, giving byte-identical builds and a \
                    stable hash preimage.",
        enforcement:
            "compile-time: the records are `const`; runtime corollary (hash determinism across \
                      calls) is checked here.",
        runtime_checkable: true,
    },
];

/// Outcome of evaluating one rule.
#[derive(Debug, Clone)]
pub struct RuleOutcome {
    pub id: &'static str,
    pub passed: bool,
    pub detail: String,
}

/// The doctrine's verdict over all five rules.
#[derive(Debug, Clone)]
pub struct SeparationVerdict {
    pub outcomes: Vec<RuleOutcome>,
    pub all_hold: bool,
}

impl SeparationVerdict {
    /// Render as plain text (one line per rule + verdict), for a CLI or report.
    pub fn render(&self) -> String {
        let mut s = String::new();
        for o in &self.outcomes {
            s.push_str(if o.passed { "  HOLDS " } else { "  BROKEN " });
            s.push_str(o.id);
            s.push_str(" — ");
            s.push_str(&o.detail);
            s.push('\n');
        }
        s.push_str(&format!(
            "verdict: {} ({}/{} rules hold)\n",
            if self.all_hold {
                "SEPARATION INTACT"
            } else {
                "SEPARATION BROKEN"
            },
            self.outcomes.iter().filter(|o| o.passed).count(),
            self.outcomes.len()
        ));
        s
    }
}

/// Evaluate the separation law against the live records. The compile-time rules (R1, R2) are reported
/// as enforced-by-construction (they could not be false in a workspace that compiled); R3/R4/R5 are
/// re-checked against the executed bank + the authority hashes.
pub fn check_separation_law() -> SeparationVerdict {
    let mut outcomes: Vec<RuleOutcome> = Vec::new();

    // R1, R2 — compile-time structural. Passed-by-construction; name the mechanism.
    for id in ["R1_authority_is_pure", "R2_execution_depends_on_authority"] {
        outcomes.push(RuleOutcome {
            id,
            passed: true,
            detail: "enforced at compile time by the crate graph (the workspace compiled)".into(),
        });
    }

    // R3 — subset: executed detectors + heuristics (and their inputs) are catalogued in the atlas.
    {
        // A small synthetic matrix instantiates build_bank (RobustZMad::fit needs samples), exactly as
        // the authority gate does. Deterministic + tiny.
        let rows: Vec<Vec<f64>> = (0..16)
            .map(|i| vec![i as f64, (i % 3) as f64, (i % 5) as f64])
            .collect();
        let m = DataMatrix::new(vec!["a".into(), "b".into(), "c".into()], rows);
        let atlas_ids: Vec<&'static str> = authority::DETECTOR_RECORDS
            .iter()
            .map(|d| d.detector_id)
            .collect();
        let mut missing: Vec<String> = Vec::new();
        for d in &build_bank(&m, 8) {
            if !atlas_ids.contains(&d.id()) {
                missing.push(format!("detector:{}", d.id()));
            }
        }
        for h in &canonical_bank() {
            if !authority::HEURISTIC_RECORDS
                .iter()
                .any(|r| r.heuristic_id == h.heuristic_id)
            {
                missing.push(format!("heuristic:{}", h.heuristic_id));
            }
            for input in &h.detector_inputs {
                if !atlas_ids.contains(&input.as_str()) {
                    missing.push(format!("heuristic_input:{}->{}", h.heuristic_id, input));
                }
            }
        }
        outcomes.push(RuleOutcome {
            id: "R3_executed_is_subset_of_catalogued",
            passed: missing.is_empty(),
            detail: if missing.is_empty() {
                "every executed detector + heuristic (and inputs) is catalogued in the atlas".into()
            } else {
                format!("uncatalogued: {missing:?}")
            },
        });
    }

    // R4 + R5 — the authority hash is deterministic + well-formed (single-sourced const records).
    {
        let a1 = authority::hashes::atlas_hash_v1();
        let a2 = authority::hashes::atlas_hash_v1();
        let atlas_ok = a1 == a2 && a1.len() == 64 && a1.bytes().all(|b| b.is_ascii_hexdigit());
        let mut detail = format!("atlas_hash_v1 deterministic + 64-hex: {atlas_ok}");

        // Corpus hash, when the optional authority is compiled in.
        #[cfg(feature = "soft-sensor-corpus")]
        let corpus_ok = {
            let c1 = dsfb_chemical_engineering_corpus::corpus_hash_v1();
            let c2 = dsfb_chemical_engineering_corpus::corpus_hash_v1();
            let ok = c1 == c2 && c1.len() == 64 && c1.bytes().all(|b| b.is_ascii_hexdigit());
            detail.push_str(&format!("; corpus_hash_v1 deterministic + 64-hex: {ok}"));
            ok
        };
        #[cfg(not(feature = "soft-sensor-corpus"))]
        let corpus_ok = {
            detail.push_str("; corpus authority not compiled in (feature off)");
            true
        };

        let pass = atlas_ok && corpus_ok;
        // R4 (one deterministic authority hash) and R5 (const records ⇒ stable hash) share this check.
        outcomes.push(RuleOutcome {
            id: "R4_one_authority_hash",
            passed: pass,
            detail: detail.clone(),
        });
        outcomes.push(RuleOutcome {
            id: "R5_static_const_records",
            passed: pass,
            detail: "authority hash is stable across calls (const `&'static` records)".into(),
        });
    }

    let all_hold = outcomes.iter().all(|o| o.passed);
    SeparationVerdict { outcomes, all_hold }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_five_separation_rules_hold() {
        // The doctrine lists exactly five rules and all of them hold against the live records.
        assert_eq!(SEPARATION_RULES.len(), 5, "the doctrine is five rules");
        let v = check_separation_law();
        assert_eq!(v.outcomes.len(), 5);
        assert!(v.all_hold, "separation law broken:\n{}", v.render());
    }
}
