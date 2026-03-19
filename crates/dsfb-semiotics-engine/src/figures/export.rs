use std::path::{Path, PathBuf};

pub fn figure_paths(figures_dir: &Path, figure_id: &str) -> (PathBuf, PathBuf) {
    (
        figures_dir.join(format!("{figure_id}.png")),
        figures_dir.join(format!("{figure_id}.svg")),
    )
}
