use crate::error::{DsfbSemiconductorError, Result};
use serde::Serialize;
use std::fs::File;
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
    pub fully_implemented: bool,
    pub blocker: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phm2018ArchiveManifest {
    pub train_sensor_files: usize,
    pub test_sensor_files: usize,
    pub train_fault_files: usize,
    pub train_ttf_files: usize,
    pub sample_paths: Vec<String>,
}

pub fn support_status(data_root: &Path) -> Phm2018SupportStatus {
    Phm2018SupportStatus {
        official_page: PHM2018_OFFICIAL_PAGE,
        official_download_link: PHM2018_DRIVE_LINK,
        expected_archive_name: PHM2018_ARCHIVE_NAME,
        manual_placement_path: data_root.join("phm2018").join(PHM2018_ARCHIVE_NAME),
        fully_implemented: false,
        blocker: "The official PHM 2018 archive is a 5.0 GB Google Drive download behind a virus-scan confirmation page. This crate provides a real archive probe and manual-placement contract, but full ingestion is not claimed unless the archive is actually present and verified.",
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
    let mut sample_paths = Vec::new();

    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?.to_string_lossy().to_string();
        if sample_paths.len() < 12 {
            sample_paths.push(path.clone());
        }
        if path.contains("/train/")
            && path.ends_with(".csv")
            && !path.contains("/train_faults/")
            && !path.contains("/train_ttf/")
        {
            train_sensor_files += 1;
        } else if path.contains("/test/") && path.ends_with(".csv") {
            test_sensor_files += 1;
        } else if path.contains("/train_faults/") && path.ends_with(".csv") {
            train_fault_files += 1;
        } else if path.contains("/train_ttf/") && path.ends_with(".csv") {
            train_ttf_files += 1;
        }
    }

    Ok(Phm2018ArchiveManifest {
        train_sensor_files,
        test_sensor_files,
        train_fault_files,
        train_ttf_files,
        sample_paths,
    })
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
    }
}
