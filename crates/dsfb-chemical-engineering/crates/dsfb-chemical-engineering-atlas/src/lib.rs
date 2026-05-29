//! `dsfb-chemical-engineering-atlas` — the chemometric/process-monitoring **authority** crate.
//!
//! DSFB-Chemical-Engineering separates *execution* from *authority*:
//! - the `edge` crate computes residual episodes,
//! - the `cuda` crate accelerates and seals evidence into replayable case files, and
//! - **this crate defines what chemometric detectors and process heuristics are *allowed to mean*.**
//!
//! It is a finite, curated, canonicalised surface: static records + deterministic validation gates +
//! SHA-256 atlas hashes. It performs **no execution** (no detectors run here; no residuals are
//! computed) and it depends on neither `edge` nor `cuda`. The `edge` crate consumes these records and
//! proves, via a subset gate, that every detector/heuristic it executes is catalogued here.
//!
//! ## Contents
//! - [`detector::ChemometricDetectorRecordV1`] + [`detector::DETECTOR_RECORDS`] — the catalogued
//!   chemometric detectors (this crate ships the 14 *executed* records as the strict authority set;
//!   the full prior-art surface remains catalogued in the edge corpus TOML).
//! - [`heuristics::ChemicalProcessHeuristicRecordV1`] + [`heuristics::HEURISTIC_RECORDS`] — H1–H6, the
//!   process-residual heuristic *records* (the rule schema as inspectable strings; the executable
//!   predicates stay in `edge`, which keeps this crate free of any execution-type dependency).
//! - [`fault_signature::FaultSignatureRecordV1`] + [`fault_signature::FAULT_SIGNATURES`] — the F1–F12
//!   process-fault signature bank: cheap-sensor residual fingerprints (stiction, cavitation, fouling,
//!   HX bypass, pump/bearing, leak, sensor drift, controller masking, valve hunting, blockage,
//!   imbalance, refrigerant), each grounded in a named public dataset; executed/catalogued honest.
//! - [`validation::validate`] — deterministic gates (id uniqueness, source-ref presence, H1–H6
//!   presence, FP/FN-mode presence, primitive-family spread).
//! - [`hashes`] — `detector_atlas_hash_v1`, `chemical_process_heuristics_hash_v1`,
//!   `challenge_docket_hash_v1`, and the composite `atlas_hash_v1`.
//!
//! ## Non-claims
//! These records assert no detector-accuracy superiority. Every process-heuristic label is a candidate
//! hypothesis for operator review with documented false-positive and false-negative modes, never a
//! proven root cause.

#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod detector;
pub mod fault_signature;
pub mod hashes;
pub mod hashing;
pub mod heuristics;
pub mod unknown_taxonomy;
pub mod validation;

pub use detector::{ChemometricDetectorRecordV1, DETECTOR_RECORDS};
pub use fault_signature::{FaultSignatureRecordV1, FAULT_SIGNATURES};
pub use heuristics::{ChemicalProcessHeuristicRecordV1, HEURISTIC_RECORDS};
pub use unknown_taxonomy::{UnknownClassRecordV1, UNKNOWN_CLASS_COUNT, UNKNOWN_TAXONOMY};
pub use validation::{validate, AtlasValidationReport};
