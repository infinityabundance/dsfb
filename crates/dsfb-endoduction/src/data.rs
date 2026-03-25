//! Dataset download, verification, and parsing for the NASA IMS bearing
//! run-to-failure dataset.
//!
//! **Provenance**: NASA Intelligent Maintenance Systems (IMS) Center,
//! University of Cincinnati. The dataset consists of run-to-failure
//! experiments on bearings under constant load. Data is publicly available
//! from the NASA Prognostics Data Repository.
//!
//! Reference: J. Lee, H. Qiu, G. Yu, J. Lin, "Rexnord Technical Services,
//! IMS, University of Cincinnati. Bearing Data Set",
//! NASA Prognostics Data Repository, 2007.

use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Description of one bearing test set within the IMS dataset.
#[derive(Debug, Clone)]
pub struct BearingSet {
    /// Set number (1, 2, or 3).
    pub set_number: u32,
    /// Number of data channels per snapshot file.
    pub channels: usize,
    /// Sampling rate in Hz.
    pub sample_rate: f64,
    /// Short description.
    pub description: &'static str,
}

/// Known bearing sets.
pub fn bearing_set_info(set: u32) -> Result<BearingSet> {
    match set {
        1 => Ok(BearingSet {
            set_number: 1,
            channels: 8,
            sample_rate: 20_000.0,
            description: "Set 1: 4 bearings, 2 accelerometers each, 2156 rpm, 6000 lb radial load",
        }),
        2 => Ok(BearingSet {
            set_number: 2,
            channels: 4,
            sample_rate: 20_000.0,
            description: "Set 2: 4 bearings, 1 accelerometer each, 2156 rpm, 6000 lb radial load",
        }),
        3 => Ok(BearingSet {
            set_number: 3,
            channels: 4,
            sample_rate: 20_000.0,
            description: "Set 3: 4 bearings, 1 accelerometer each, 2156 rpm, 6000 lb radial load",
        }),
        _ => bail!("Unknown bearing set {set}. Valid: 1, 2, 3"),
    }
}

/// A single parsed snapshot: one file from the bearing dataset.
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// File name (for provenance).
    pub file_name: String,
    /// Chronological index.
    pub index: usize,
    /// Per-channel sample vectors.
    pub channels: Vec<Vec<f64>>,
}

/// Locate the dataset directory for a given bearing set.
///
/// Expects the directory structure:
///   `<data_root>/IMS/<set_dir>/`
/// where `set_dir` is one of `1st_test`, `2nd_test`, `3rd_test`.
///
/// Note: In some IMS distributions, set 3 is packaged as `4th_test/txt/`.
/// This function checks alternative paths automatically.
pub fn dataset_dir(data_root: &Path, set: u32) -> PathBuf {
    let ims = data_root.join("IMS");
    let primary = match set {
        1 => ims.join("1st_test"),
        2 => ims.join("2nd_test"),
        3 => ims.join("3rd_test"),
        _ => ims.join("1st_test"),
    };
    if primary.is_dir() {
        return primary;
    }
    // Handle known alternative layouts from the IMS archive.
    if set == 3 {
        // Some archives extract set 3 as "4th_test/txt/" or just "4th_test/".
        let alt1 = ims.join("4th_test").join("txt");
        if alt1.is_dir() {
            return alt1;
        }
        let alt2 = ims.join("4th_test");
        if alt2.is_dir() {
            return alt2;
        }
    }
    // Return primary even if it doesn't exist; verify_dataset will error.
    primary
}

/// Check whether the dataset directory exists and contains snapshot files.
pub fn verify_dataset(dir: &Path) -> Result<Vec<String>> {
    if !dir.is_dir() {
        bail!(
            "Dataset directory does not exist: {}. \
             Download the NASA IMS bearing dataset and extract it.",
            dir.display()
        );
    }
    let mut files: Vec<String> = fs::read_dir(dir)
        .with_context(|| format!("Cannot read dataset dir {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    files.sort();
    if files.is_empty() {
        bail!("Dataset directory is empty: {}", dir.display());
    }
    Ok(files)
}

/// Parse a single snapshot file.
///
/// NASA IMS files are tab-separated or space-separated ASCII with one row per
/// sample and one column per channel.
pub fn parse_snapshot(path: &Path, expected_channels: usize, index: usize) -> Result<Snapshot> {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let f = fs::File::open(path).with_context(|| format!("Cannot open {}", path.display()))?;
    let reader = BufReader::new(f);
    let mut channels: Vec<Vec<f64>> = vec![Vec::new(); expected_channels];
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let vals: Vec<f64> = trimmed
            .split(|c: char| c == '\t' || c == ' ')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<f64>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("Parse error in {}", path.display()))?;
        let ncols = vals.len().min(expected_channels);
        for (ch, &v) in vals.iter().take(ncols).enumerate() {
            channels[ch].push(v);
        }
    }
    Ok(Snapshot {
        file_name,
        index,
        channels,
    })
}

/// Load all snapshots from a bearing set in chronological order.
///
/// **Warning**: This holds all channels in memory. For large datasets,
/// prefer [`load_channel_data`] which extracts only the required channel.
pub fn load_all_snapshots(data_root: &Path, set: u32) -> Result<Vec<Snapshot>> {
    let info = bearing_set_info(set)?;
    let dir = dataset_dir(data_root, set);
    let files = verify_dataset(&dir)?;
    eprintln!(
        "[data] Loading {} snapshots from {} ({})",
        files.len(),
        dir.display(),
        info.description
    );
    let mut snapshots = Vec::with_capacity(files.len());
    for (i, fname) in files.iter().enumerate() {
        let path = dir.join(fname);
        let snap = parse_snapshot(&path, info.channels, i)?;
        snapshots.push(snap);
    }
    eprintln!("[data] Loaded {} snapshots successfully.", snapshots.len());
    Ok(snapshots)
}

/// Memory-efficient channel extraction: parse each snapshot file and keep
/// only the single requested channel. This avoids holding all channels
/// of all snapshots in memory simultaneously.
///
/// Returns `(file_names, channel_data)` where `channel_data[i]` is the
/// signal vector for snapshot `i` on the requested channel.
pub fn load_channel_data(
    data_root: &Path,
    set: u32,
    channel: usize,
) -> Result<(Vec<String>, Vec<Vec<f64>>)> {
    let info = bearing_set_info(set)?;
    if channel >= info.channels {
        bail!(
            "Requested channel {} but set {} has only {} channels",
            channel,
            set,
            info.channels
        );
    }
    let dir = dataset_dir(data_root, set);
    let files = verify_dataset(&dir)?;
    let total = files.len();
    eprintln!(
        "[data] Loading channel {} from {} snapshots in {} ({})",
        channel,
        total,
        dir.display(),
        info.description
    );

    let mut names = Vec::with_capacity(total);
    let mut signals = Vec::with_capacity(total);

    for (i, fname) in files.iter().enumerate() {
        let path = dir.join(fname);
        let snap = parse_snapshot(&path, info.channels, i)?;
        let ch_data = if channel < snap.channels.len() {
            snap.channels[channel].clone()
        } else {
            bail!(
                "Snapshot {} has {} channels, expected at least {}",
                fname,
                snap.channels.len(),
                channel + 1
            );
        };
        names.push(snap.file_name);
        signals.push(ch_data);
        // snap (with all channels) is dropped here, freeing memory.

        if (i + 1) % 500 == 0 {
            eprintln!("[data] Loaded {}/{} snapshots...", i + 1, total);
        }
    }

    eprintln!("[data] Loaded {} snapshots successfully.", total);
    Ok((names, signals))
}

/// Extract one channel's data from a snapshot as a flat vector.
pub fn extract_channel(snap: &Snapshot, channel: usize) -> Result<&[f64]> {
    snap.channels
        .get(channel)
        .map(|v| v.as_slice())
        .with_context(|| {
            format!(
                "Channel {} not available in snapshot {} (has {} channels)",
                channel,
                snap.file_name,
                snap.channels.len()
            )
        })
}

/// Download the NASA IMS bearing dataset.
///
/// This function downloads the dataset archive from a public mirror and
/// extracts it. It uses the NASA Prognostics Data Repository hosted on
/// a public mirror.
pub fn download_dataset(data_root: &Path) -> Result<()> {
    let ims_dir = data_root.join("IMS");
    if ims_dir.is_dir() {
        let entries: Vec<_> = fs::read_dir(&ims_dir)?
            .filter_map(|e| e.ok())
            .collect();
        if !entries.is_empty() {
            eprintln!("[data] IMS dataset already present at {}", ims_dir.display());
            return Ok(());
        }
    }
    fs::create_dir_all(&ims_dir)?;

    // The NASA IMS dataset is publicly available from the NASA Prognostics
    // Center of Excellence. The original data.nasa.gov link is no longer
    // reliable, so we try multiple known mirrors in order.
    //
    // Provenance: J. Lee, H. Qiu, G. Yu, J. Lin, "Rexnord Technical Services,
    // IMS, University of Cincinnati. Bearing Data Set", NASA Prognostics Data
    // Repository, 2007.
    let mirrors: &[&str] = &[
        // PHM Society dataset mirror (commonly used in literature)
        "https://phm-datasets.s3.amazonaws.com/NASA/4.+Bearings.zip",
        // NASA TI/ARC redirect
        "https://ti.arc.nasa.gov/c/6/",
        // data.nasa.gov (may be intermittently available)
        "https://data.nasa.gov/download/brfb-gzcv/application%2Fx-zip-compressed",
    ];
    let archive_path = data_root.join("IMS.zip");

    let mut downloaded = false;
    for (i, &url) in mirrors.iter().enumerate() {
        eprintln!(
            "[data] Attempting download from mirror {}/{}: {}",
            i + 1,
            mirrors.len(),
            url
        );

        // Try curl first.
        let status = std::process::Command::new("curl")
            .args(["-L", "-o"])
            .arg(archive_path.as_os_str())
            .arg(url)
            .args(["--progress-bar", "--fail", "--connect-timeout", "30"])
            .status();

        if let Ok(s) = status {
            if s.success() && archive_path.exists() && fs::metadata(&archive_path).map(|m| m.len() > 1_000_000).unwrap_or(false) {
                eprintln!("[data] Download succeeded from mirror {}.", i + 1);
                downloaded = true;
                break;
            }
        }

        // Try wget as fallback for this mirror.
        let status2 = std::process::Command::new("wget")
            .args(["-O"])
            .arg(archive_path.as_os_str())
            .arg(url)
            .args(["--progress=bar:force", "--timeout=30"])
            .status();

        if let Ok(s) = status2 {
            if s.success() && archive_path.exists() && fs::metadata(&archive_path).map(|m| m.len() > 1_000_000).unwrap_or(false) {
                eprintln!("[data] Download succeeded from mirror {} (wget).", i + 1);
                downloaded = true;
                break;
            }
        }

        eprintln!("[data] Mirror {} failed, trying next...", i + 1);
        // Remove partial download before trying next mirror.
        let _ = fs::remove_file(&archive_path);
    }

    if !downloaded {
        bail!(
            "Dataset download failed from all mirrors. Please download manually:\n\
             1. https://phm-datasets.s3.amazonaws.com/NASA/4.+Bearings.zip\n\
             2. https://www.nasa.gov/content/prognostics-center-of-excellence-data-set-repository\n\
             \n\
             Then extract the contents so that the directory structure is:\n\
             {}/1st_test/  (containing snapshot files)\n\
             {}/2nd_test/\n\
             {}/3rd_test/",
            ims_dir.display(),
            ims_dir.display(),
            ims_dir.display()
        );
    }

    eprintln!("[data] Extracting archive...");
    // The dataset is a ZIP containing inner archives or directories.
    extract_ims_archive(&archive_path, data_root)?;
    eprintln!("[data] Dataset extraction complete.");
    Ok(())
}

/// Extract the IMS dataset archive (ZIP format with nested structure).
fn extract_ims_archive(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)
        .with_context(|| format!("Cannot open archive {}", archive_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Invalid ZIP archive: {}", archive_path.display()))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        let out_path = dest.join(&name);
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }
    }

    // The IMS dataset has a deeply nested archive structure:
    //   PHM mirror:  ZIP → "4. Bearings/IMS.7z" → {1st_test.rar, 2nd_test.rar, 3rd_test.rar}
    //   Each .rar contains the actual snapshot files for that test set.
    let ims_dir = dest.join("IMS");

    // Phase 1: Extract any nested .7z archives.
    if !ims_dir.join("1st_test").is_dir() {
        for entry in walkdir(dest)? {
            let fname = entry
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();

            if fname.ends_with(".7z") && entry.is_file() {
                eprintln!("[data] Found nested 7z archive: {}", entry.display());
                fs::create_dir_all(&ims_dir)?;

                let extract_ok = try_7z_extract(&entry, &ims_dir)?;
                if !extract_ok {
                    eprintln!("[data] 7z not found. Attempting to install p7zip-full...");
                    let _ = std::process::Command::new("sudo")
                        .args(["apt-get", "install", "-y", "p7zip-full"])
                        .status();
                    let _ = std::process::Command::new("apt-get")
                        .args(["install", "-y", "p7zip-full"])
                        .status();

                    if !try_7z_extract(&entry, &ims_dir)? {
                        bail!(
                            "Cannot extract 7z archive. Install p7zip-full:\n\
                             sudo apt-get install p7zip-full\n\
                             Then re-run, or manually extract {} into {}",
                            entry.display(),
                            ims_dir.display()
                        );
                    }
                }
                break;
            }
        }
    }

    // Phase 2: Extract any nested .rar archives inside IMS/.
    if ims_dir.is_dir() && !ims_dir.join("1st_test").is_dir() {
        let rar_files: Vec<PathBuf> = walkdir(&ims_dir)?
            .into_iter()
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .map(|e| e.to_string_lossy().to_lowercase() == "rar")
                        .unwrap_or(false)
            })
            .collect();

        if !rar_files.is_empty() {
            eprintln!("[data] Found {} nested RAR archive(s). Extracting...", rar_files.len());

            // Ensure unrar is available.
            let has_unrar = std::process::Command::new("unrar")
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if !has_unrar {
                eprintln!("[data] unrar not found. Attempting to install...");
                let _ = std::process::Command::new("sudo")
                    .args(["apt-get", "install", "-y", "unrar"])
                    .status();
                let _ = std::process::Command::new("apt-get")
                    .args(["install", "-y", "unrar"])
                    .status();
            }

            for rar in &rar_files {
                eprintln!("[data] Extracting: {}", rar.display());
                let status = std::process::Command::new("unrar")
                    .args(["x", "-y", "-o+"])
                    .arg(rar.as_os_str())
                    .arg(ims_dir.as_os_str())
                    .status()
                    .with_context(|| format!("Failed to extract {}", rar.display()))?;
                if !status.success() {
                    // Try 7z as fallback for RAR extraction.
                    let status2 = std::process::Command::new("7z")
                        .args(["x", "-y"])
                        .arg(format!("-o{}", ims_dir.display()))
                        .arg(rar.as_os_str())
                        .status();
                    if !status2.map(|s| s.success()).unwrap_or(false) {
                        bail!(
                            "Cannot extract RAR archive {}. Install unrar:\n\
                             sudo apt-get install unrar",
                            rar.display()
                        );
                    }
                }
            }
        }
    }

    // Phase 3: Find and relocate test directories if needed.
    if !ims_dir.join("1st_test").is_dir() {
        for entry in walkdir(dest)? {
            let fname = entry
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if fname == "1st_test" && entry.is_dir() {
                if let Some(parent) = entry.parent() {
                    if parent != ims_dir {
                        if ims_dir.exists() {
                            fs::remove_dir_all(&ims_dir)?;
                        }
                        fs::rename(parent, &ims_dir).ok();
                    }
                }
                break;
            }
        }
    }

    Ok(())
}

/// Try to extract a 7z archive using available system tools.
/// Returns Ok(true) if extraction succeeded, Ok(false) if no tool found.
fn try_7z_extract(archive: &Path, dest: &Path) -> Result<bool> {
    // Try `7z x` first (p7zip-full).
    let status = std::process::Command::new("7z")
        .args(["x", "-y"])
        .arg(format!("-o{}", dest.display()))
        .arg(archive.as_os_str())
        .status();

    if let Ok(s) = status {
        if s.success() {
            return Ok(true);
        }
    }

    // Try `7zr` (p7zip minimal).
    let status2 = std::process::Command::new("7zr")
        .args(["x", "-y"])
        .arg(format!("-o{}", dest.display()))
        .arg(archive.as_os_str())
        .status();

    if let Ok(s) = status2 {
        if s.success() {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Simple recursive directory walk.
fn walkdir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            results.push(path.clone());
            if path.is_dir() {
                results.extend(walkdir(&path)?);
            }
        }
    }
    Ok(results)
}
