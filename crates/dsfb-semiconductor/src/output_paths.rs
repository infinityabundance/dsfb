use chrono::Local;
use std::path::{Path, PathBuf};

pub fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn repo_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("crate directory to live under <repo>/crates/dsfb-semiconductor")
}

pub fn default_data_root() -> PathBuf {
    crate_root().join("data").join("raw")
}

pub fn default_output_root() -> PathBuf {
    repo_root().join("output-dsfb-semiconductor")
}

pub fn create_timestamped_run_dir(output_root: &Path, dataset: &str) -> std::io::Result<PathBuf> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
    let base = format!("{}_{}", timestamp, dataset);
    for attempt in 0..1000 {
        let candidate = if attempt == 0 {
            output_root.join(&base)
        } else {
            output_root.join(format!("{base}_{attempt:03}"))
        };
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!(
            "failed to allocate unique run directory under {}",
            output_root.display()
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_root_targets_repo_level_directory() {
        let output_root = default_output_root();
        assert_eq!(
            output_root.file_name().unwrap(),
            "output-dsfb-semiconductor"
        );
    }

    #[test]
    fn timestamped_dir_contains_dataset_suffix() {
        let temp = tempfile::tempdir().unwrap();
        let run_dir = create_timestamped_run_dir(temp.path(), "secom").unwrap();
        let name = run_dir.file_name().unwrap().to_string_lossy();
        assert!(name.ends_with("_secom"));
    }

    #[test]
    fn timestamped_dir_never_reuses_existing_directory_name() {
        let temp = tempfile::tempdir().unwrap();
        let first = create_timestamped_run_dir(temp.path(), "secom").unwrap();
        let second = create_timestamped_run_dir(temp.path(), "secom").unwrap();
        assert_ne!(first, second);
    }
}
