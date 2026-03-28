// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// NASA PCoE battery cell helpers for additive multi-cell workflows.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct NasaPcoeCellSpec {
    pub cell_id: &'static str,
    pub csv_filename: &'static str,
}

const FY08Q4_CELLS: [NasaPcoeCellSpec; 4] = [
    NasaPcoeCellSpec {
        cell_id: "B0005",
        csv_filename: "nasa_b0005_capacity.csv",
    },
    NasaPcoeCellSpec {
        cell_id: "B0006",
        csv_filename: "nasa_b0006_capacity.csv",
    },
    NasaPcoeCellSpec {
        cell_id: "B0007",
        csv_filename: "nasa_b0007_capacity.csv",
    },
    NasaPcoeCellSpec {
        cell_id: "B0018",
        csv_filename: "nasa_b0018_capacity.csv",
    },
];

pub fn supported_nasa_pcoe_cells() -> &'static [NasaPcoeCellSpec] {
    &FY08Q4_CELLS
}

pub fn default_nasa_cell_csv_path(data_dir: &Path, cell: &NasaPcoeCellSpec) -> PathBuf {
    data_dir.join(cell.csv_filename)
}
