use crate::error::{DsfbSemiconductorError, Result};
use chrono::NaiveDateTime;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub const SECOM_UCI_URL: &str = "https://archive.ics.uci.edu/static/public/179/secom.zip";
pub const SECOM_ARCHIVE_NAME: &str = "secom.zip";
pub const SECOM_DATA_FILE: &str = "secom.data";
pub const SECOM_LABELS_FILE: &str = "secom_labels.data";
pub const SECOM_NAMES_FILE: &str = "secom.names";

#[derive(Debug, Clone, Serialize)]
pub struct SecomRun {
    pub index: usize,
    pub label: i8,
    pub timestamp: NaiveDateTime,
    pub features: Vec<Option<f64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecomDataset {
    pub feature_names: Vec<String>,
    pub runs: Vec<SecomRun>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecomDataPaths {
    pub root: PathBuf,
    pub archive: PathBuf,
    pub data_file: PathBuf,
    pub labels_file: PathBuf,
    pub names_file: PathBuf,
}

pub fn dataset_paths(data_root: &Path) -> SecomDataPaths {
    let root = data_root.join("secom");
    SecomDataPaths {
        archive: root.join(SECOM_ARCHIVE_NAME),
        data_file: root.join(SECOM_DATA_FILE),
        labels_file: root.join(SECOM_LABELS_FILE),
        names_file: root.join(SECOM_NAMES_FILE),
        root,
    }
}

pub fn fetch_if_missing(data_root: &Path) -> Result<SecomDataPaths> {
    let paths = dataset_paths(data_root);
    if paths.data_file.exists() && paths.labels_file.exists() && paths.names_file.exists() {
        return Ok(paths);
    }

    fs::create_dir_all(&paths.root)?;

    if !paths.archive.exists() {
        let response = ureq::get(SECOM_UCI_URL)
            .call()
            .map_err(|err| DsfbSemiconductorError::Network(err.to_string()))?;
        let mut reader = response.into_reader();
        let mut file = File::create(&paths.archive)?;
        std::io::copy(&mut reader, &mut file)?;
    }

    unpack_archive(&paths.archive, &paths.root)?;
    Ok(paths)
}

pub fn ensure_present(data_root: &Path) -> Result<SecomDataPaths> {
    let paths = dataset_paths(data_root);
    if paths.data_file.exists() && paths.labels_file.exists() && paths.names_file.exists() {
        Ok(paths)
    } else {
        Err(DsfbSemiconductorError::DatasetMissing {
            dataset: "SECOM",
            path: paths.root,
        })
    }
}

pub fn load_from_root(data_root: &Path) -> Result<SecomDataset> {
    let paths = ensure_present(data_root)?;
    load_from_paths(&paths)
}

pub fn load_from_paths(paths: &SecomDataPaths) -> Result<SecomDataset> {
    let labels = read_labels(&paths.labels_file)?;
    let data = read_data(&paths.data_file)?;

    if labels.len() != data.len() {
        return Err(DsfbSemiconductorError::DatasetFormat(format!(
            "SECOM rows do not match labels: {} data rows vs {} labels",
            data.len(),
            labels.len()
        )));
    }

    let feature_count = data.first().map(Vec::len).unwrap_or_default();
    let feature_names = (1..=feature_count)
        .map(|idx| format!("S{idx:03}"))
        .collect::<Vec<_>>();

    let runs = data
        .into_iter()
        .zip(labels.into_iter())
        .enumerate()
        .map(|(index, (features, (label, timestamp)))| SecomRun {
            index,
            label,
            timestamp,
            features,
        })
        .collect::<Vec<_>>();

    Ok(SecomDataset {
        feature_names,
        runs,
    })
}

fn unpack_archive(archive_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let out_path = output_dir.join(entry.name());
        let mut out_file = File::create(out_path)?;
        let mut buffer = Vec::new();
        entry.read_to_end(&mut buffer)?;
        out_file.write_all(&buffer)?;
    }
    Ok(())
}

fn read_labels(path: &Path) -> Result<Vec<(i8, NaiveDateTime)>> {
    let reader = BufReader::new(File::open(path)?);
    let mut labels = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.splitn(2, ' ');
        let label = parts
            .next()
            .ok_or_else(|| DsfbSemiconductorError::DatasetFormat("missing SECOM label".into()))?
            .parse::<i8>()
            .map_err(|err| DsfbSemiconductorError::DatasetFormat(err.to_string()))?;
        let timestamp_raw = parts
            .next()
            .ok_or_else(|| {
                DsfbSemiconductorError::DatasetFormat("missing SECOM label timestamp".into())
            })?
            .trim_matches('"');
        let timestamp = NaiveDateTime::parse_from_str(timestamp_raw, "%d/%m/%Y %H:%M:%S")
            .map_err(|err| DsfbSemiconductorError::DatasetFormat(err.to_string()))?;
        labels.push((label, timestamp));
    }

    Ok(labels)
}

fn read_data(path: &Path) -> Result<Vec<Vec<Option<f64>>>> {
    let reader = BufReader::new(File::open(path)?);
    let mut rows = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let row = trimmed
            .split_whitespace()
            .map(|token| {
                if token.eq_ignore_ascii_case("nan") {
                    Ok(None)
                } else {
                    token.parse::<f64>().map(Some).map_err(|err| {
                        DsfbSemiconductorError::DatasetFormat(format!(
                            "invalid SECOM value `{token}`: {err}"
                        ))
                    })
                }
            })
            .collect::<Result<Vec<_>>>()?;
        rows.push(row);
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_parse_from_uci_format() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("labels.data");
        fs::write(
            &path,
            "-1 \"19/07/2008 11:55:00\"\n1 \"19/07/2008 13:17:00\"\n",
        )
        .unwrap();
        let labels = read_labels(&path).unwrap();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].0, -1);
        assert_eq!(labels[1].0, 1);
    }

    #[test]
    fn data_parser_keeps_nan_as_missing() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("secom.data");
        fs::write(&path, "1.0 NaN 2.5\n").unwrap();
        let rows = read_data(&path).unwrap();
        assert_eq!(rows[0], vec![Some(1.0), None, Some(2.5)]);
    }
}
