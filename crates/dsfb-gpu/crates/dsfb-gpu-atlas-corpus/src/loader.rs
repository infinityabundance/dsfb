//! TOML-AST → corpus record loader.
//!
//! Converts the [`crate::toml_parser::DetectorRecord`] AST into
//! owned-data [`LoadedLiteratureDetector`] values. The owned variant
//! mirrors [`crate::types::LiteratureDetector`] field-for-field but
//! uses `String` and `Vec` in place of `&'static str` and `&'static
//! [T]` so the corpus can be assembled at runtime from on-disk TOML.
//!
//! Equivalence with the static seed is the load-bearing T.2
//! contract:
//!
//! - `LoadedLiteratureDetector::matches_static(&LiteratureDetector)`
//!   provides field-by-field byte equivalence.
//! - The T.2 equivalence tests dump the static seed, parse the
//!   resulting TOML, and assert every loaded record matches its
//!   static counterpart.
//!
//! T.3+ may move the loaded variant onto `&'static` storage via a
//! one-shot leak helper so `verify` / `report` can run on TOML-loaded
//! data without API churn; T.2 keeps the two paths cleanly separated.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryFrom;

use crate::toml_parser::{parse_detectors, DetectorRecord, Value};
use crate::types::{
    AxisBindingSet, ConfuserProfile, ConstitutionFlags, DecisionFunctional, DetectorCanonicalId,
    DeterministicStatus, DomainTagSet, DuplicateGroupId, GpuFamilyKernel, ImplementationLevel,
    InputRequirementSet, LifecycleState, LiteratureDetector, MathFormId, NegativeWitnessKind,
    PrimitiveFamily, UsefulnessLedgerSnapshot, WitnessKind, WitnessRole,
};

/// Owned-data variant of [`LiteratureDetector`]. Identical shape
/// with `String` / `Vec` in place of `&'static str` / `&'static [T]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedLiteratureDetector {
    /// Canonical record handle.
    pub canonical_id: DetectorCanonicalId,
    /// Human-readable display name.
    pub display_name: String,
    /// Other names this primitive goes by.
    pub aliases: Vec<String>,
    /// Provenance records.
    pub source_refs: Vec<LoadedSourceRef>,
    /// Domain bitset.
    pub origin_domains: DomainTagSet,
    /// Primitive-family classification.
    pub primitive_family: PrimitiveFamily,
    /// Coarse mathematical-form classification.
    pub mathematical_form: MathFormId,
    /// Decision-functional shape.
    pub decision_functional: DecisionFunctional,
    /// Input-contract bitset.
    pub input_requirements: InputRequirementSet,
    /// Output witness type.
    pub output_witness: WitnessKind,
    /// Witness role in the fusion court.
    pub witness_role: WitnessRole,
    /// Negative-witness sub-classifier.
    pub negative_witness_kind: NegativeWitnessKind,
    /// Fusion-axis bitset.
    pub fusion_axes: AxisBindingSet,
    /// Confuser-profile classification.
    pub confuser_profile: ConfuserProfile,
    /// Deterministic-status classification.
    pub deterministic_status: DeterministicStatus,
    /// Implementation-status band.
    pub implementation_status: ImplementationLevel,
    /// GPU execution family.
    pub gpu_family: GpuFamilyKernel,
    /// Parameter-bound descriptor.
    pub parameter_bounds: LoadedParameterBounds,
    /// Duplicate-equivalence class.
    pub duplicate_group: DuplicateGroupId,
    /// Genealogy edges.
    pub genealogy: LoadedGenealogyEdges,
    /// Usefulness-ledger row (unmeasured at T.2; T.8 populates).
    pub usefulness: UsefulnessLedgerSnapshot,
    /// Lifecycle state.
    pub lifecycle_state: LifecycleState,
    /// Constitution flags.
    pub constitution_compliance: ConstitutionFlags,
}

/// Owned-data variant of [`crate::types::SourceRef`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSourceRef {
    /// Short citation key.
    pub citation_key: String,
    /// Title.
    pub title: String,
    /// Authors (canonical-form string).
    pub authors: String,
    /// Year of publication.
    pub year: u16,
    /// Venue / journal / publisher.
    pub venue_or_source: String,
    /// Optional DOI / URL.
    pub doi_or_url: Option<String>,
    /// Free-form provenance note.
    pub notes: String,
}

/// Owned-data variant of [`crate::types::ParameterBounds`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedParameterBounds {
    /// Number of independent parameter axes.
    pub axis_count: u8,
    /// Human-readable description.
    pub description: String,
}

/// Owned-data variant of [`crate::types::GenealogyEdges`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedGenealogyEdges {
    /// Direct ancestors.
    pub derived_from: Vec<DetectorCanonicalId>,
    /// Primitives this one generalises.
    pub generalizes: Vec<DetectorCanonicalId>,
    /// Primitives this one is a special case of.
    pub special_case_of: Vec<DetectorCanonicalId>,
    /// True if this is an origin record.
    pub is_origin: bool,
}

/// Loader failure with the record context and reason.
#[derive(Debug, Clone)]
pub struct LoadError {
    /// 1-based detector index in the source (the Nth `[[detector]]`
    /// block). 0 indicates a non-record-specific failure (e.g.
    /// `parse_detectors` errored out before any record was opened).
    pub record_index: usize,
    /// Field / context where the failure occurred.
    pub context: String,
    /// Reason classifier.
    pub reason: LoadErrorReason,
}

/// Why a record could not be loaded.
#[derive(Debug, Clone)]
pub enum LoadErrorReason {
    /// Underlying TOML parser failed.
    ParseError(String),
    /// A required field is missing from the record.
    MissingField(String),
    /// A field has the wrong TOML type.
    WrongType {
        /// Expected type name.
        expected: &'static str,
        /// What was found.
        found: &'static str,
    },
    /// An enum wire name didn't match any known variant.
    UnknownEnum {
        /// Enum type name.
        enum_name: &'static str,
        /// Offending wire string.
        value: String,
    },
    /// A bitset flag name didn't match any known bit.
    UnknownBit {
        /// Bitset type name.
        set_name: &'static str,
        /// Offending wire string.
        value: String,
    },
    /// Integer out of range for the target field.
    OutOfRange(String),
}

impl LoadError {
    /// Format as a single-line diagnostic string.
    #[must_use]
    pub fn display(&self) -> String {
        format!(
            "record #{}: `{}` -> {:?}",
            self.record_index, self.context, self.reason
        )
    }
}

/// Parse the TOML source and convert every `[[detector]]` block into
/// an owned [`LoadedLiteratureDetector`]. The records are returned in
/// source order.
///
/// # Errors
/// Returns the first record-conversion failure. The parser-level
/// errors propagate as [`LoadErrorReason::ParseError`] with
/// `record_index = 0`.
pub fn load_from_str(input: &str) -> Result<Vec<LoadedLiteratureDetector>, LoadError> {
    let parsed = parse_detectors(input).map_err(|e| LoadError {
        record_index: 0,
        context: String::from("toml_parser"),
        reason: LoadErrorReason::ParseError(e.display()),
    })?;
    let mut out = Vec::with_capacity(parsed.len());
    for (i, rec) in parsed.iter().enumerate() {
        let loaded = load_one_record(rec, i + 1)?;
        out.push(loaded);
    }
    Ok(out)
}

fn wrap_err(record_index: usize, context: &str, reason: LoadErrorReason) -> LoadError {
    LoadError {
        record_index,
        context: context.to_string(),
        reason,
    }
}

fn load_one_record(
    rec: &DetectorRecord,
    record_index: usize,
) -> Result<LoadedLiteratureDetector, LoadError> {
    let canonical_id = u32::try_from(
        require_int(rec, "canonical_id").map_err(|r| wrap_err(record_index, "canonical_id", r))?,
    )
    .map_err(|_| LoadError {
        record_index,
        context: "canonical_id".to_string(),
        reason: LoadErrorReason::OutOfRange("canonical_id must fit in u32".to_string()),
    })?;
    let display_name = require_string(rec, "display_name")
        .map_err(|r| wrap_err(record_index, "display_name", r))?;
    let aliases =
        require_string_array(rec, "aliases").map_err(|r| wrap_err(record_index, "aliases", r))?;
    let primitive_family =
        require_enum::<PrimitiveFamily>(rec, "primitive_family", "PrimitiveFamily")
            .map_err(|r| wrap_err(record_index, "primitive_family", r))?;
    let mathematical_form = require_enum::<MathFormId>(rec, "mathematical_form", "MathFormId")
        .map_err(|r| wrap_err(record_index, "mathematical_form", r))?;
    let decision_functional =
        require_enum::<DecisionFunctional>(rec, "decision_functional", "DecisionFunctional")
            .map_err(|r| wrap_err(record_index, "decision_functional", r))?;
    let input_requirements = require_input_requirement_set(rec)
        .map_err(|r| wrap_err(record_index, "input_requirements", r))?;
    let origin_domains =
        require_domain_tag_set(rec).map_err(|r| wrap_err(record_index, "origin_domains", r))?;
    let output_witness = require_enum::<WitnessKind>(rec, "output_witness", "WitnessKind")
        .map_err(|r| wrap_err(record_index, "output_witness", r))?;
    let witness_role = require_enum::<WitnessRole>(rec, "witness_role", "WitnessRole")
        .map_err(|r| wrap_err(record_index, "witness_role", r))?;
    let negative_witness_kind =
        require_enum::<NegativeWitnessKind>(rec, "negative_witness_kind", "NegativeWitnessKind")
            .map_err(|r| wrap_err(record_index, "negative_witness_kind", r))?;
    let fusion_axes =
        require_axis_binding_set(rec).map_err(|r| wrap_err(record_index, "fusion_axes", r))?;
    let confuser_profile =
        require_enum::<ConfuserProfile>(rec, "confuser_profile", "ConfuserProfile")
            .map_err(|r| wrap_err(record_index, "confuser_profile", r))?;
    let deterministic_status =
        require_enum::<DeterministicStatus>(rec, "deterministic_status", "DeterministicStatus")
            .map_err(|r| wrap_err(record_index, "deterministic_status", r))?;
    let implementation_status =
        require_enum::<ImplementationLevel>(rec, "implementation_status", "ImplementationLevel")
            .map_err(|r| wrap_err(record_index, "implementation_status", r))?;
    let gpu_family = require_enum::<GpuFamilyKernel>(rec, "gpu_family", "GpuFamilyKernel")
        .map_err(|r| wrap_err(record_index, "gpu_family", r))?;
    let parameter_bounds =
        require_parameter_bounds(rec).map_err(|r| wrap_err(record_index, "parameter_bounds", r))?;
    let duplicate_group = u32::try_from(
        require_int(rec, "duplicate_group")
            .map_err(|r| wrap_err(record_index, "duplicate_group", r))?,
    )
    .map_err(|_| LoadError {
        record_index,
        context: "duplicate_group".to_string(),
        reason: LoadErrorReason::OutOfRange("duplicate_group must fit in u32".to_string()),
    })?;
    let genealogy = require_genealogy(rec).map_err(|r| wrap_err(record_index, "genealogy", r))?;
    let lifecycle_state = require_enum::<LifecycleState>(rec, "lifecycle_state", "LifecycleState")
        .map_err(|r| wrap_err(record_index, "lifecycle_state", r))?;
    let constitution_compliance = require_constitution_flags(rec)
        .map_err(|r| wrap_err(record_index, "constitution_compliance", r))?;
    let source_refs =
        require_source_refs(rec).map_err(|r| wrap_err(record_index, "source_refs", r))?;

    Ok(LoadedLiteratureDetector {
        canonical_id: DetectorCanonicalId(canonical_id),
        display_name,
        aliases,
        source_refs,
        origin_domains,
        primitive_family,
        mathematical_form,
        decision_functional,
        input_requirements,
        output_witness,
        witness_role,
        negative_witness_kind,
        fusion_axes,
        confuser_profile,
        deterministic_status,
        implementation_status,
        gpu_family,
        parameter_bounds,
        duplicate_group: DuplicateGroupId(duplicate_group),
        genealogy,
        usefulness: UsefulnessLedgerSnapshot::unmeasured(),
        lifecycle_state,
        constitution_compliance,
    })
}

// =========================================================
// Helpers
// =========================================================

fn require_int(rec: &DetectorRecord, name: &str) -> Result<i64, LoadErrorReason> {
    let v = rec
        .fields
        .get(name)
        .ok_or_else(|| LoadErrorReason::MissingField(name.to_string()))?;
    match v {
        Value::Int(i) => Ok(*i),
        other => Err(LoadErrorReason::WrongType {
            expected: "int",
            found: value_kind(other),
        }),
    }
}

fn require_string(rec: &DetectorRecord, name: &str) -> Result<String, LoadErrorReason> {
    let v = rec
        .fields
        .get(name)
        .ok_or_else(|| LoadErrorReason::MissingField(name.to_string()))?;
    match v {
        Value::String(s) => Ok(s.clone()),
        other => Err(LoadErrorReason::WrongType {
            expected: "string",
            found: value_kind(other),
        }),
    }
}

fn require_string_array(rec: &DetectorRecord, name: &str) -> Result<Vec<String>, LoadErrorReason> {
    let v = rec
        .fields
        .get(name)
        .ok_or_else(|| LoadErrorReason::MissingField(name.to_string()))?;
    match v {
        Value::Array(items) => items
            .iter()
            .map(|x| match x {
                Value::String(s) => Ok(s.clone()),
                other => Err(LoadErrorReason::WrongType {
                    expected: "string in array",
                    found: value_kind(other),
                }),
            })
            .collect(),
        other => Err(LoadErrorReason::WrongType {
            expected: "array",
            found: value_kind(other),
        }),
    }
}

fn require_enum<E: EnumWire>(
    rec: &DetectorRecord,
    name: &str,
    enum_name: &'static str,
) -> Result<E, LoadErrorReason> {
    let s = require_string(rec, name)?;
    E::from_wire(&s).ok_or(LoadErrorReason::UnknownEnum {
        enum_name,
        value: s,
    })
}

trait EnumWire: Sized {
    fn from_wire(s: &str) -> Option<Self>;
}

macro_rules! impl_enum_wire {
    ($t:ty) => {
        impl EnumWire for $t {
            fn from_wire(s: &str) -> Option<Self> {
                <$t>::from_wire(s)
            }
        }
    };
}
impl_enum_wire!(PrimitiveFamily);
impl_enum_wire!(MathFormId);
impl_enum_wire!(DecisionFunctional);
impl_enum_wire!(WitnessKind);
impl_enum_wire!(WitnessRole);
impl_enum_wire!(NegativeWitnessKind);
impl_enum_wire!(ConfuserProfile);
impl_enum_wire!(DeterministicStatus);
impl_enum_wire!(ImplementationLevel);
impl_enum_wire!(LifecycleState);
impl_enum_wire!(GpuFamilyKernel);

fn require_input_requirement_set(
    rec: &DetectorRecord,
) -> Result<InputRequirementSet, LoadErrorReason> {
    let names = require_string_array(rec, "input_requirements")?;
    let mut bits = 0u32;
    for n in &names {
        bits |=
            InputRequirementSet::bit_from_wire(n).ok_or_else(|| LoadErrorReason::UnknownBit {
                set_name: "InputRequirementSet",
                value: n.clone(),
            })?;
    }
    Ok(InputRequirementSet(bits))
}

fn require_axis_binding_set(rec: &DetectorRecord) -> Result<AxisBindingSet, LoadErrorReason> {
    let names = require_string_array(rec, "fusion_axes")?;
    let mut bits = 0u16;
    for n in &names {
        bits |= AxisBindingSet::bit_from_wire(n).ok_or_else(|| LoadErrorReason::UnknownBit {
            set_name: "AxisBindingSet",
            value: n.clone(),
        })?;
    }
    Ok(AxisBindingSet(bits))
}

fn require_domain_tag_set(rec: &DetectorRecord) -> Result<DomainTagSet, LoadErrorReason> {
    let names = require_string_array(rec, "origin_domains")?;
    let mut bits = 0u16;
    for n in &names {
        bits |= DomainTagSet::bit_from_wire(n).ok_or_else(|| LoadErrorReason::UnknownBit {
            set_name: "DomainTagSet",
            value: n.clone(),
        })?;
    }
    Ok(DomainTagSet(bits))
}

fn require_parameter_bounds(
    rec: &DetectorRecord,
) -> Result<LoadedParameterBounds, LoadErrorReason> {
    let table = rec
        .subtables
        .get("parameter_bounds")
        .ok_or_else(|| LoadErrorReason::MissingField("[detector.parameter_bounds]".to_string()))?;
    let axis_count_int = table
        .get("axis_count")
        .ok_or_else(|| LoadErrorReason::MissingField("parameter_bounds.axis_count".to_string()))
        .and_then(|v| match v {
            Value::Int(i) => Ok(*i),
            other => Err(LoadErrorReason::WrongType {
                expected: "int",
                found: value_kind(other),
            }),
        })?;
    let axis_count = u8::try_from(axis_count_int).map_err(|_| {
        LoadErrorReason::OutOfRange("parameter_bounds.axis_count must fit in u8".to_string())
    })?;
    let description = table
        .get("description")
        .ok_or_else(|| LoadErrorReason::MissingField("parameter_bounds.description".to_string()))
        .and_then(|v| match v {
            Value::String(s) => Ok(s.clone()),
            other => Err(LoadErrorReason::WrongType {
                expected: "string",
                found: value_kind(other),
            }),
        })?;
    Ok(LoadedParameterBounds {
        axis_count,
        description,
    })
}

fn require_genealogy(rec: &DetectorRecord) -> Result<LoadedGenealogyEdges, LoadErrorReason> {
    let table = rec
        .subtables
        .get("genealogy")
        .ok_or_else(|| LoadErrorReason::MissingField("[detector.genealogy]".to_string()))?;
    let derived_from = table_int_array_to_ids(table, "derived_from")?;
    let generalizes = table_int_array_to_ids(table, "generalizes")?;
    let special_case_of = table_int_array_to_ids(table, "special_case_of")?;
    let is_origin = table
        .get("is_origin")
        .ok_or_else(|| LoadErrorReason::MissingField("genealogy.is_origin".to_string()))
        .and_then(|v| match v {
            Value::Bool(b) => Ok(*b),
            other => Err(LoadErrorReason::WrongType {
                expected: "bool",
                found: value_kind(other),
            }),
        })?;
    Ok(LoadedGenealogyEdges {
        derived_from,
        generalizes,
        special_case_of,
        is_origin,
    })
}

fn table_int_array_to_ids(
    table: &alloc::collections::BTreeMap<String, Value>,
    key: &str,
) -> Result<Vec<DetectorCanonicalId>, LoadErrorReason> {
    let v = table
        .get(key)
        .ok_or_else(|| LoadErrorReason::MissingField(format!("genealogy.{key}")))?;
    match v {
        Value::Array(items) => items
            .iter()
            .map(|x| match x {
                Value::Int(i) => u32::try_from(*i)
                    .map(DetectorCanonicalId)
                    .map_err(|_| LoadErrorReason::OutOfRange(format!("genealogy.{key} entry"))),
                other => Err(LoadErrorReason::WrongType {
                    expected: "int in array",
                    found: value_kind(other),
                }),
            })
            .collect(),
        other => Err(LoadErrorReason::WrongType {
            expected: "array",
            found: value_kind(other),
        }),
    }
}

fn require_constitution_flags(rec: &DetectorRecord) -> Result<ConstitutionFlags, LoadErrorReason> {
    let table = rec
        .subtables
        .get("constitution_compliance")
        .ok_or_else(|| {
            LoadErrorReason::MissingField("[detector.constitution_compliance]".to_string())
        })?;
    let get = |key: &str| -> Result<bool, LoadErrorReason> {
        let v = table.get(key).ok_or_else(|| {
            LoadErrorReason::MissingField(format!("constitution_compliance.{key}"))
        })?;
        match v {
            Value::Bool(b) => Ok(*b),
            other => Err(LoadErrorReason::WrongType {
                expected: "bool",
                found: value_kind(other),
            }),
        }
    };
    Ok(ConstitutionFlags {
        declared_input_contract: get("declared_input_contract")?,
        declared_output_type: get("declared_output_type")?,
        declared_deterministic_form: get("declared_deterministic_form")?,
        declared_provenance: get("declared_provenance")?,
        declared_equivalence_status: get("declared_equivalence_status")?,
        declared_witness_role: get("declared_witness_role")?,
        declared_activation_conditions: get("declared_activation_conditions")?,
        declared_failure_confuser_modes: get("declared_failure_confuser_modes")?,
    })
}

fn require_source_refs(rec: &DetectorRecord) -> Result<Vec<LoadedSourceRef>, LoadErrorReason> {
    let arr = rec
        .array_subtables
        .get("source_refs")
        .ok_or_else(|| LoadErrorReason::MissingField("[[detector.source_refs]]".to_string()))?;
    let mut out = Vec::with_capacity(arr.len());
    for entry in arr {
        let get_str = |k: &str| -> Result<String, LoadErrorReason> {
            let v = entry
                .get(k)
                .ok_or_else(|| LoadErrorReason::MissingField(format!("source_refs.{k}")))?;
            match v {
                Value::String(s) => Ok(s.clone()),
                other => Err(LoadErrorReason::WrongType {
                    expected: "string",
                    found: value_kind(other),
                }),
            }
        };
        let year_i = entry
            .get("year")
            .ok_or_else(|| LoadErrorReason::MissingField("source_refs.year".to_string()))
            .and_then(|v| match v {
                Value::Int(i) => Ok(*i),
                other => Err(LoadErrorReason::WrongType {
                    expected: "int",
                    found: value_kind(other),
                }),
            })?;
        let year = u16::try_from(year_i)
            .map_err(|_| LoadErrorReason::OutOfRange("source_refs.year".to_string()))?;
        let doi_or_url = match entry.get("doi_or_url") {
            None => None,
            Some(Value::String(s)) if s.is_empty() => None,
            Some(Value::String(s)) => Some(s.clone()),
            Some(other) => {
                return Err(LoadErrorReason::WrongType {
                    expected: "string or empty string",
                    found: value_kind(other),
                });
            }
        };
        out.push(LoadedSourceRef {
            citation_key: get_str("citation_key")?,
            title: get_str("title")?,
            authors: get_str("authors")?,
            year,
            venue_or_source: get_str("venue_or_source")?,
            doi_or_url,
            notes: get_str("notes")?,
        });
    }
    Ok(out)
}

fn value_kind(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "string",
        Value::Int(_) => "int",
        Value::Bool(_) => "bool",
        Value::Array(_) => "array",
    }
}

// =========================================================
// Equivalence helpers — used by the T.2 acceptance tests to
// compare a static-seed record against a loaded record.
// =========================================================

impl LoadedLiteratureDetector {
    /// True if every field equals the static [`LiteratureDetector`]
    /// byte-for-byte (modulo owned-vs-borrowed storage).
    #[must_use]
    pub fn matches_static(&self, sref: &LiteratureDetector) -> bool {
        self.canonical_id == sref.canonical_id
            && self.display_name == sref.display_name
            && slices_eq_str(&self.aliases, sref.aliases)
            && source_refs_match(&self.source_refs, sref.source_refs)
            && self.origin_domains == sref.origin_domains
            && self.primitive_family == sref.primitive_family
            && self.mathematical_form == sref.mathematical_form
            && self.decision_functional == sref.decision_functional
            && self.input_requirements == sref.input_requirements
            && self.output_witness == sref.output_witness
            && self.witness_role == sref.witness_role
            && self.negative_witness_kind == sref.negative_witness_kind
            && self.fusion_axes == sref.fusion_axes
            && self.confuser_profile == sref.confuser_profile
            && self.deterministic_status == sref.deterministic_status
            && self.implementation_status == sref.implementation_status
            && self.gpu_family == sref.gpu_family
            && self.parameter_bounds.axis_count == sref.parameter_bounds.axis_count
            && self.parameter_bounds.description == sref.parameter_bounds.description
            && self.duplicate_group == sref.duplicate_group
            && genealogy_matches(&self.genealogy, &sref.genealogy)
            && self.lifecycle_state == sref.lifecycle_state
            && self.constitution_compliance == sref.constitution_compliance
    }
}

fn slices_eq_str(loaded: &[String], stat: &[&'static str]) -> bool {
    loaded.len() == stat.len() && loaded.iter().zip(stat).all(|(a, b)| a == b)
}

fn source_refs_match(loaded: &[LoadedSourceRef], stat: &[crate::types::SourceRef]) -> bool {
    if loaded.len() != stat.len() {
        return false;
    }
    for (l, s) in loaded.iter().zip(stat) {
        if l.citation_key != s.citation_key
            || l.title != s.title
            || l.authors != s.authors
            || l.year != s.year
            || l.venue_or_source != s.venue_or_source
            || l.doi_or_url.as_deref() != s.doi_or_url
            || l.notes != s.notes
        {
            return false;
        }
    }
    true
}

fn genealogy_matches(loaded: &LoadedGenealogyEdges, stat: &crate::types::GenealogyEdges) -> bool {
    loaded.is_origin == stat.is_origin
        && slices_eq_ids(&loaded.derived_from, stat.derived_from)
        && slices_eq_ids(&loaded.generalizes, stat.generalizes)
        && slices_eq_ids(&loaded.special_case_of, stat.special_case_of)
}

fn slices_eq_ids(loaded: &[DetectorCanonicalId], stat: &[DetectorCanonicalId]) -> bool {
    loaded.len() == stat.len() && loaded.iter().zip(stat).all(|(a, b)| a == b)
}
