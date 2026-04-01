use crate::error::{DsfbSemiconductorError, Result};
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};

pub const PHM2018_OFFICIAL_PAGE: &str = "https://phmsociety.org/conference/annual-conference-of-the-phm-society/annual-conference-of-the-prognostics-and-health-management-society-2018-b/phm-data-challenge-6/";
pub const PHM2018_DRIVE_LINK: &str =
    "https://drive.google.com/open?id=15Jx9Scq9FqpIGn8jbAQB_lcHSXvIoPzb";
pub const PHM2018_ARCHIVE_NAME: &str = "phm_data_challenge_2018.tar.gz";

#[derive(Debug, Clone, Serialize)]
pub struct Phm2018SupportStatus {
    pub official_page: &'static str,
    pub official_download_link: &'static str,
    pub expected_archive_name: &'static str,
    pub manual_placement_path: PathBuf,
    pub archive_summary_supported: bool,
    pub fully_implemented: bool,
    pub blocker: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phm2018CsvShape {
    pub path: String,
    pub column_count: usize,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phm2018ArchiveManifest {
    pub train_sensor_files: usize,
    pub test_sensor_files: usize,
    pub train_fault_files: usize,
    pub train_ttf_files: usize,
    pub train_sensor_schema: Phm2018CsvGroupSummary,
    pub test_sensor_schema: Phm2018CsvGroupSummary,
    pub train_fault_schema: Phm2018CsvGroupSummary,
    pub train_ttf_schema: Phm2018CsvGroupSummary,
    pub schema_note: String,
    pub sample_paths: Vec<String>,
    pub sample_csv_shapes: Vec<Phm2018CsvShape>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Phm2018CsvGroupSummary {
    pub file_count: usize,
    pub distinct_column_counts: Vec<usize>,
    pub sampled_headers: Vec<Vec<String>>,
}

pub fn support_status(data_root: &Path) -> Phm2018SupportStatus {
    Phm2018SupportStatus {
        official_page: PHM2018_OFFICIAL_PAGE,
        official_download_link: PHM2018_DRIVE_LINK,
        expected_archive_name: PHM2018_ARCHIVE_NAME,
        manual_placement_path: data_root.join("phm2018").join(PHM2018_ARCHIVE_NAME),
        archive_summary_supported: true,
        fully_implemented: false,
        blocker: "The official PHM 2018 archive is a 5.0 GB Google Drive download behind a virus-scan confirmation page. This crate now provides a deterministic archive probe, grouped CSV-schema summary, and CSV-shape ingestion summary, but a full DSFB benchmark path is still not claimed unless the real archive is present and schema-verified end to end.",
    }
}

pub fn inspect_archive(archive_path: &Path) -> Result<Phm2018ArchiveManifest> {
    let file = File::open(archive_path).map_err(|_| DsfbSemiconductorError::DatasetMissing {
        dataset: "PHM 2018 ion mill etch",
        path: archive_path.to_path_buf(),
    })?;

    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    let mut train_sensor_files = 0usize;
    let mut test_sensor_files = 0usize;
    let mut train_fault_files = 0usize;
    let mut train_ttf_files = 0usize;
    let mut train_sensor_schema = Phm2018CsvGroupSummary::default();
    let mut test_sensor_schema = Phm2018CsvGroupSummary::default();
    let mut train_fault_schema = Phm2018CsvGroupSummary::default();
    let mut train_ttf_schema = Phm2018CsvGroupSummary::default();
    let mut sample_paths = Vec::new();
    let mut sample_csv_shapes = Vec::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().to_string();
        if sample_paths.len() < 12 {
            sample_paths.push(path.clone());
        }
        if !path.ends_with(".csv") {
            continue;
        }

        let class = classify_csv_path(&path);
        let (header, sample_shape) = if sample_csv_shapes.len() < 8 {
            let bytes = entry_bytes(&mut entry)?;
            let (shape, header) = csv_shape_and_header(&path, &bytes)?;
            sample_csv_shapes.push(shape);
            (header, true)
        } else {
            (csv_header(&mut entry)?, false)
        };
        let _ = sample_shape;

        match class {
            CsvClass::TrainSensor => {
                train_sensor_files += 1;
                update_group_summary(&mut train_sensor_schema, &header);
            }
            CsvClass::TestSensor => {
                test_sensor_files += 1;
                update_group_summary(&mut test_sensor_schema, &header);
            }
            CsvClass::TrainFault => {
                train_fault_files += 1;
                update_group_summary(&mut train_fault_schema, &header);
            }
            CsvClass::TrainTtf => {
                train_ttf_files += 1;
                update_group_summary(&mut train_ttf_schema, &header);
            }
            CsvClass::Other => {}
        }
    }

    let schema_note = format!(
        "Train/test sensor schemas are {} and {}. Fault/TTF sidecar schemas are {} and {}.",
        schema_consistency_note(&train_sensor_schema),
        schema_consistency_note(&test_sensor_schema),
        schema_consistency_note(&train_fault_schema),
        schema_consistency_note(&train_ttf_schema),
    );

    Ok(Phm2018ArchiveManifest {
        train_sensor_files,
        test_sensor_files,
        train_fault_files,
        train_ttf_files,
        train_sensor_schema,
        test_sensor_schema,
        train_fault_schema,
        train_ttf_schema,
        schema_note,
        sample_paths,
        sample_csv_shapes,
    })
}

#[derive(Debug, Clone, Copy)]
enum CsvClass {
    TrainSensor,
    TestSensor,
    TrainFault,
    TrainTtf,
    Other,
}

fn classify_csv_path(path: &str) -> CsvClass {
    if path.contains("/train/") && !path.contains("/train_faults/") && !path.contains("/train_ttf/")
    {
        CsvClass::TrainSensor
    } else if path.contains("/test/") {
        CsvClass::TestSensor
    } else if path.contains("/train_faults/") {
        CsvClass::TrainFault
    } else if path.contains("/train_ttf/") {
        CsvClass::TrainTtf
    } else {
        CsvClass::Other
    }
}

fn update_group_summary(summary: &mut Phm2018CsvGroupSummary, header: &[String]) {
    summary.file_count += 1;
    let width = header.len();
    if !summary.distinct_column_counts.contains(&width) {
        summary.distinct_column_counts.push(width);
        summary.distinct_column_counts.sort_unstable();
    }
    if summary.sampled_headers.len() < 3 {
        summary.sampled_headers.push(header.to_vec());
    }
}

fn schema_consistency_note(summary: &Phm2018CsvGroupSummary) -> String {
    if summary.file_count == 0 {
        "not present".into()
    } else if summary.distinct_column_counts.len() == 1 {
        format!(
            "column-consistent at width {}",
            summary.distinct_column_counts[0]
        )
    } else {
        format!("mixed column widths {:?}", summary.distinct_column_counts)
    }
}

fn entry_bytes(entry: &mut tar::Entry<'_, flate2::read::GzDecoder<File>>) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn csv_shape_and_header(path: &str, bytes: &[u8]) -> Result<(Phm2018CsvShape, Vec<String>)> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes);
    let headers = reader
        .headers()?
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let mut row_count = 0usize;
    for record in reader.records() {
        record?;
        row_count += 1;
    }
    Ok((
        Phm2018CsvShape {
            path: path.to_string(),
            column_count: headers.len(),
            row_count,
        },
        headers,
    ))
}

fn csv_header(entry: &mut tar::Entry<'_, flate2::read::GzDecoder<File>>) -> Result<Vec<String>> {
    let mut line = String::new();
    let mut reader = std::io::BufReader::new(entry);
    reader.read_line(&mut line)?;
    if line.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(line.as_bytes());
    let record = csv_reader
        .records()
        .next()
        .transpose()?
        .unwrap_or_else(csv::StringRecord::new);
    Ok(record.iter().map(|value| value.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tar::Builder;

    #[test]
    fn support_status_points_to_manual_archive_location() {
        let status = support_status(Path::new("/tmp/dsfb-semiconductor-data"));
        assert!(status
            .manual_placement_path
            .ends_with("phm2018/phm_data_challenge_2018.tar.gz"));
        assert!(!status.fully_implemented);
    }

    #[test]
    fn archive_probe_counts_expected_csv_classes() {
        let temp = tempfile::tempdir().unwrap();
        let archive_path = temp.path().join("phm_data_challenge_2018.tar.gz");

        let file = File::create(&archive_path).unwrap();
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(encoder);

        fn append_csv(builder: &mut Builder<GzEncoder<File>>, path: &str, content: &[u8]) {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, path, content).unwrap();
        }

        append_csv(&mut builder, "challenge/train/run_001.csv", b"a,b\n1,2\n");
        append_csv(&mut builder, "challenge/train/run_002.csv", b"a,b\n3,4\n");
        append_csv(&mut builder, "challenge/test/run_003.csv", b"a,b\n5,6\n");
        append_csv(
            &mut builder,
            "challenge/train_faults/run_001.csv",
            b"fault\n1\n",
        );
        append_csv(
            &mut builder,
            "challenge/train_ttf/run_001.csv",
            b"ttf\n10\n",
        );
        builder.finish().unwrap();
        let mut encoder = builder.into_inner().unwrap();
        encoder.flush().unwrap();

        let manifest = inspect_archive(&archive_path).unwrap();
        assert_eq!(manifest.train_sensor_files, 2);
        assert_eq!(manifest.test_sensor_files, 1);
        assert_eq!(manifest.train_fault_files, 1);
        assert_eq!(manifest.train_ttf_files, 1);
        assert!(!manifest.sample_paths.is_empty());
        assert!(!manifest.sample_csv_shapes.is_empty());
        assert_eq!(manifest.sample_csv_shapes[0].column_count, 2);
        assert_eq!(manifest.train_sensor_schema.file_count, 2);
        assert_eq!(manifest.test_sensor_schema.file_count, 1);
        assert_eq!(manifest.train_sensor_schema.distinct_column_counts, vec![2]);
        assert!(manifest.schema_note.contains("column-consistent"));
    }
}
