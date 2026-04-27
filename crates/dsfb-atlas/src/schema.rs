//! YAML schema for atlas Part specifications.
//!
//! Each `PNN_*.yaml` file under `crates/dsfb-bank/spec/atlas/`
//! deserialises into a [`Part`] containing 10 [`Chapter`] entries; each
//! chapter carries 10 method `stems` and 10 modifier strings, producing
//! the 1,000-theorem-per-Part shape that the generator emits.

use serde::{Deserialize, Serialize};

/// One reduction-lens Part of the atlas (P01..P10).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Part {
    /// Part identifier such as `P01`.
    pub part_id: String,
    /// Human-readable Part name (e.g. "Categorical").
    pub part_name: String,
    /// Reduction-lens identifier (snake-case; e.g. `categorical`).
    pub lens: String,
    /// Default empirical-anchor tier for chapters that do not override it.
    pub default_anchor_tier: String,
    /// Default missing-pipeline-layer list inherited by chapters.
    pub missing_layers_default: Vec<String>,
    /// Default reduction-kind (`constructive`, `existential`, `isomorphic`,
    /// or `bisimilar`) inherited by chapters.
    pub reduction_kind_default: String,
    /// Default LaTeX class colour name inherited by chapters.
    pub default_class_color: String,
    /// 10 chapters per Part.
    pub chapters: Vec<Chapter>,
}

/// One method-family Chapter inside a Part.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Chapter {
    /// Chapter identifier such as `P01-C03`.
    pub chapter_id: String,
    /// Human-readable chapter name.
    pub chapter_name: String,
    /// Optional per-chapter LaTeX class colour override.
    #[serde(default)]
    pub class_color: Option<String>,
    /// Optional per-chapter empirical-anchor-tier override.
    #[serde(default)]
    pub anchor_tier: Option<String>,
    /// Bank theorem identifiers (e.g. `CORE-02`) cited as T1 anchors.
    #[serde(default)]
    pub anchor_bank_ids: Vec<String>,
    /// Optional paperstack-paper citation for T2 anchors.
    #[serde(default)]
    pub paperstack_cite: Option<String>,
    /// Optional public-dataset accession for T3 anchors.
    #[serde(default)]
    pub public_dataset: Option<String>,
    /// Optional per-chapter missing-pipeline-layer override.
    #[serde(default)]
    pub missing_layers: Option<Vec<String>>,
    /// Optional per-chapter reduction-kind override.
    #[serde(default)]
    pub reduction_kind: Option<String>,
    /// Method stems (10 per chapter).
    pub stems: Vec<String>,
    /// Method modifiers (10 per chapter).
    pub modifiers: Vec<String>,
    /// LaTeX template for the method operation phrase, with `{stem}` and
    /// `{modifier}` placeholders.
    pub operation_phrase_template: String,
    /// Free-form output-type description used in atlas-theorem statements.
    pub output_type: String,
    /// Free-form input-signal-class description used in atlas-theorem
    /// statements.
    pub input_signal_class: String,
}

impl Chapter {
    /// Resolve the effective class colour, falling back to `default` when
    /// the chapter does not override it.
    #[must_use]
    pub fn effective_class_color<'a>(&'a self, default: &'a str) -> &'a str {
        self.class_color.as_deref().unwrap_or(default)
    }

    /// Resolve the effective empirical-anchor tier.
    #[must_use]
    pub fn effective_anchor_tier<'a>(&'a self, default: &'a str) -> &'a str {
        self.anchor_tier.as_deref().unwrap_or(default)
    }

    /// Resolve the effective missing-pipeline-layer list.
    #[must_use]
    pub fn effective_missing_layers<'a>(&'a self, default: &'a [String]) -> &'a [String] {
        match &self.missing_layers {
            Some(v) => v.as_slice(),
            None => default,
        }
    }

    /// Resolve the effective reduction-kind.
    #[must_use]
    pub fn effective_reduction_kind<'a>(&'a self, default: &'a str) -> &'a str {
        self.reduction_kind.as_deref().unwrap_or(default)
    }
}
