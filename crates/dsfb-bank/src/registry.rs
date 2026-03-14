use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Component {
    Core,
    Dsfb,
    Dscd,
    Tmtr,
    Add,
    Srd,
    Hret,
}

impl Component {
    pub const ALL: [Component; 7] = [
        Component::Core,
        Component::Dsfb,
        Component::Dscd,
        Component::Tmtr,
        Component::Add,
        Component::Srd,
        Component::Hret,
    ];

    pub const BANKS: [Component; 6] = [
        Component::Dsfb,
        Component::Dscd,
        Component::Tmtr,
        Component::Add,
        Component::Srd,
        Component::Hret,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Component::Core => "core",
            Component::Dsfb => "dsfb",
            Component::Dscd => "dscd",
            Component::Tmtr => "tmtr",
            Component::Add => "add",
            Component::Srd => "srd",
            Component::Hret => "hret",
        }
    }

    pub fn spec_filename(self) -> Option<&'static str> {
        match self {
            Component::Core => Some("core_theorems.yaml"),
            Component::Dsfb => Some("dsfb_theorems.yaml"),
            Component::Dscd => Some("dscd_theorems.yaml"),
            Component::Tmtr => Some("tmtr_theorems.yaml"),
            Component::Add => Some("add_theorems.yaml"),
            Component::Srd => Some("srd_theorems.yaml"),
            Component::Hret => Some("hret_theorems.yaml"),
        }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TheoremSpec {
    pub id: String,
    pub component: Component,
    pub ordinal: u32,
    pub title: String,
    pub statement_summary: String,
    pub assumptions: Vec<String>,
    pub variables: Vec<String>,
    pub expected_behavior: Vec<String>,
    pub witness_cases: Vec<String>,
    pub runner: String,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RealizationSpec {
    pub component: Component,
    pub realization_name: String,
    pub category: String,
    pub operator_domain: String,
    pub operator_codomain: String,
    pub notes: String,
    pub empirical_status: String,
}

#[derive(Debug, Clone)]
pub struct TheoremRegistry {
    theorem_banks: BTreeMap<Component, Vec<TheoremSpec>>,
    realizations: Vec<RealizationSpec>,
}

impl TheoremRegistry {
    pub fn load() -> Result<Self> {
        let mut theorem_banks = BTreeMap::new();
        for component in Component::ALL {
            let specs = load_theorem_specs(component)?;
            theorem_banks.insert(component, specs);
        }
        let realizations = load_realizations()?;
        Ok(Self {
            theorem_banks,
            realizations,
        })
    }

    pub fn all_theorems(&self) -> Vec<&TheoremSpec> {
        let mut items = Vec::new();
        for component in Component::ALL {
            if let Some(specs) = self.theorem_banks.get(&component) {
                for spec in specs {
                    items.push(spec);
                }
            }
        }
        items
    }

    pub fn theorems_for(&self, component: Component) -> &[TheoremSpec] {
        self.theorem_banks
            .get(&component)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn bank_components(&self) -> &'static [Component] {
        &Component::BANKS
    }

    pub fn realizations_for(&self, component: Component) -> Vec<RealizationSpec> {
        self.realizations
            .iter()
            .filter(|spec| spec.component == component)
            .cloned()
            .collect()
    }

    pub fn all_realizations(&self) -> &[RealizationSpec] {
        &self.realizations
    }
}

pub fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

pub fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(Path::parent)
        .expect("crate lives in workspace/crates/dsfb-bank")
        .to_path_buf()
}

pub fn load_theorem_specs(component: Component) -> Result<Vec<TheoremSpec>> {
    let filename = component
        .spec_filename()
        .expect("every component used by the registry has a spec file");
    let path = crate_root().join("spec").join(filename);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read theorem specs at {}", path.display()))?;
    let mut specs: Vec<TheoremSpec> = serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse theorem specs at {}", path.display()))?;
    specs.sort_by_key(|spec| spec.ordinal);
    Ok(specs)
}

pub fn load_realizations() -> Result<Vec<RealizationSpec>> {
    let path = crate_root().join("spec").join("realizations.yaml");
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read realizations at {}", path.display()))?;
    let specs: Vec<RealizationSpec> = serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse realizations at {}", path.display()))?;
    Ok(specs)
}
