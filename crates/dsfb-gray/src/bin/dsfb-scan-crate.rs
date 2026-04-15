use dsfb_gray::{
    export_scan_artifacts, migrate_legacy_scan_artifacts, prepare_scan_output_run,
    render_scan_report, scan_crate_source, scan_crate_source_with_profile, CrateSourceScanReport,
    ScanProfile, ScanSigningKey, DEFAULT_SCAN_OUTPUT_ROOT,
};
use std::env;
use std::io::{self, IsTerminal};
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

type CliArgIter = Peekable<std::iter::Skip<env::ArgsOs>>;

const SCAN_SOURCES_DIR: &str = "target/scan-sources";
const USER_AGENT: &str = "dsfb-scan-crate/0.1.0 (+https://crates.io/crates/dsfb-gray)";

fn main() {
    let style = Style::detect();
    let args = parse_cli_args();

    print_banner(&style);

    let root = match resolve_scan_root(&args.root, &style) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("  {} {}", style.red("✗ error"), err);
            process::exit(1);
        }
    };

    eprintln!(
        "  {} Scanning {}",
        style.cyan("[scan]"),
        style.bold(&root.display().to_string())
    );
    eprintln!();

    let scan_result = match args.legacy_profile {
        Some(profile) => scan_crate_source_with_profile(&root, profile),
        None => scan_crate_source(&root),
    };

    if args.legacy_profile.is_some() {
        eprintln!(
            "  {} --profile is deprecated; DSFB now emits one canonical broad audit.",
            style.yellow("[note]")
        );
    }

    match scan_result {
        Ok(report) => {
            let rendered_report = render_scan_report(&report);
            print!("{rendered_report}");
            let base_output_root = args
                .out_dir
                .unwrap_or_else(|| PathBuf::from(DEFAULT_SCAN_OUTPUT_ROOT));
            export_report_artifacts(&report, &base_output_root, &style);
        }
        Err(err) => {
            eprintln!(
                "  {} failed to scan {}: {}",
                style.red("✗ error"),
                root.display(),
                err
            );
            process::exit(1);
        }
    }
}

struct CliArgs {
    out_dir: Option<PathBuf>,
    legacy_profile: Option<ScanProfile>,
    root: PathBuf,
}

fn parse_cli_args() -> CliArgs {
    let mut args = env::args_os().skip(1).peekable();
    let mut out_dir = None;
    let mut legacy_profile = None;
    let mut root = None;

    while let Some(arg) = args.next() {
        if arg == "--out-dir" {
            out_dir = Some(next_path_arg(&mut args, "--out-dir"));
        } else if arg == "--profile" {
            legacy_profile = Some(parse_legacy_profile_arg(&mut args));
        } else if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if root.is_none() {
            root = Some(PathBuf::from(arg));
        } else {
            print_usage_and_exit();
        }
    }

    CliArgs {
        out_dir,
        legacy_profile,
        root: root.unwrap_or_else(|| print_usage_and_exit()),
    }
}

fn next_path_arg(args: &mut CliArgIter, flag: &str) -> PathBuf {
    let Some(value) = args.next() else {
        eprintln!("error: {flag} requires a directory argument");
        process::exit(2);
    };
    PathBuf::from(value)
}

fn parse_legacy_profile_arg(args: &mut CliArgIter) -> ScanProfile {
    let Some(value) = args.next() else {
        eprintln!("error: --profile requires a profile name");
        process::exit(2);
    };
    let value = value.to_string_lossy();
    let Some(parsed) = ScanProfile::parse(&value) else {
        eprintln!(
            "error: unsupported profile `{}` (use general, cloud-native, distributed-systems, industrial-safety, or supply-chain)",
            value
        );
        process::exit(2);
    };
    parsed
}

fn print_usage() {
    eprintln!(
        "\
usage: cargo run --bin dsfb-scan-crate -- [--out-dir DIR] <path-or-crate-name>

  <path-or-crate-name>
      Either a local directory containing Rust crate source, OR a crate name
      to fetch from crates.io. Crate-name downloads are cached in
      target/scan-sources/<name>-<version>/ and scanned from there.

  --out-dir DIR
      Override the base output directory (default: output-dsfb-gray/).
      A dsfb-gray-<UTC-timestamp>/ run subdirectory is always created inside.

Examples:
  cargo run --bin dsfb-scan-crate -- ./path/to/my-crate
  cargo run --bin dsfb-scan-crate -- base64
  cargo run --bin dsfb-scan-crate -- dsfb-battery"
    );
}

fn print_usage_and_exit() -> ! {
    print_usage();
    process::exit(2);
}

fn resolve_scan_root(input: &Path, style: &Style) -> io::Result<PathBuf> {
    if input.is_dir() {
        eprintln!(
            "  {} Local source: {}",
            style.cyan("[path]"),
            style.dim(&input.display().to_string())
        );
        return Ok(input.to_path_buf());
    }

    let input_str = input.to_string_lossy();
    if looks_like_path(&input_str) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a directory", input.display()),
        ));
    }
    if !is_valid_crate_name(&input_str) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{} is neither an existing directory nor a valid crates.io crate name",
                input.display()
            ),
        ));
    }

    fetch_and_extract_from_crates_io(&input_str, style)
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/')
        || s.contains('\\')
        || s.starts_with('.')
        || s.starts_with('~')
        || Path::new(s).is_absolute()
}

fn is_valid_crate_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn fetch_and_extract_from_crates_io(name: &str, style: &Style) -> io::Result<PathBuf> {
    eprintln!(
        "  {} Resolving {} on crates.io",
        style.cyan("[1/3]"),
        style.bold(name)
    );
    let version = fetch_latest_version(name).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!(
                "could not resolve `{}` on crates.io ({}). Is the crate name correct?",
                name, err
            ),
        )
    })?;
    eprintln!("         version: {}", style.green(&version));

    let scan_sources = PathBuf::from(SCAN_SOURCES_DIR);
    std::fs::create_dir_all(&scan_sources)?;
    let crate_dir = scan_sources.join(format!("{name}-{version}"));

    if crate_dir.is_dir() {
        eprintln!(
            "  {} Cached source: {}",
            style.cyan("[2/3]"),
            style.dim(&crate_dir.display().to_string())
        );
        eprintln!(
            "  {} {}",
            style.cyan("[3/3]"),
            style.dim("(skip extract — using cached tree)")
        );
        return Ok(crate_dir);
    }

    let tarball = scan_sources.join(format!("{name}-{version}.crate"));
    eprintln!(
        "  {} Downloading {} {}",
        style.cyan("[2/3]"),
        style.bold(name),
        style.bold(&version)
    );
    download_tarball(name, &version, &tarball)?;
    let size_kb = std::fs::metadata(&tarball)
        .map(|m| m.len() / 1024)
        .unwrap_or(0);
    eprintln!(
        "         → {} ({} KB)",
        style.dim(&tarball.display().to_string()),
        size_kb
    );

    eprintln!(
        "  {} Extracting to {}",
        style.cyan("[3/3]"),
        style.dim(&crate_dir.display().to_string())
    );
    extract_tarball(&tarball, &scan_sources)?;

    if !crate_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "tarball extracted but expected directory {} was not created",
                crate_dir.display()
            ),
        ));
    }

    Ok(crate_dir)
}

fn fetch_latest_version(name: &str) -> io::Result<String> {
    let url = format!("https://crates.io/api/v1/crates/{name}");
    let body = curl_get(&url)?;
    let parsed: serde_json::Value = serde_json::from_str(&body).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("crates.io returned unparseable JSON: {err}"),
        )
    })?;
    parsed
        .get("crate")
        .and_then(|c| c.get("max_version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "crates.io response did not include crate.max_version",
            )
        })
}

fn curl_get(url: &str) -> io::Result<String> {
    let output = Command::new("curl")
        .args(["-sSL", "--fail", "-A", USER_AGENT, url])
        .output()
        .map_err(|err| io::Error::new(err.kind(), format!("failed to invoke curl: {err}")))?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "curl failed for {} (exit {}): {}",
                url,
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
}

fn download_tarball(name: &str, version: &str, dest: &Path) -> io::Result<()> {
    let url = format!("https://crates.io/api/v1/crates/{name}/{version}/download");
    let status = Command::new("curl")
        .args(["-sSL", "--fail", "-A", USER_AGENT, "-o"])
        .arg(dest)
        .arg(&url)
        .status()
        .map_err(|err| io::Error::new(err.kind(), format!("failed to invoke curl: {err}")))?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "curl download failed for {} (exit {})",
                url,
                status.code().unwrap_or(-1)
            ),
        ));
    }
    Ok(())
}

fn extract_tarball(tarball: &Path, dest_dir: &Path) -> io::Result<()> {
    let status = Command::new("tar")
        .arg("xzf")
        .arg(tarball)
        .arg("-C")
        .arg(dest_dir)
        .status()
        .map_err(|err| io::Error::new(err.kind(), format!("failed to invoke tar: {err}")))?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "tar extraction failed for {} (exit {})",
                tarball.display(),
                status.code().unwrap_or(-1)
            ),
        ));
    }
    Ok(())
}

fn export_report_artifacts(
    report: &CrateSourceScanReport,
    base_output_root: &Path,
    style: &Style,
) {
    let cwd = match env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!(
                "  {} failed to resolve current directory: {err}",
                style.red("✗ error")
            );
            process::exit(1);
        }
    };
    let migrated_legacy_dir = match migrate_legacy_scan_artifacts(&cwd, base_output_root) {
        Ok(path) => path,
        Err(err) => {
            eprintln!(
                "  {} failed to migrate legacy scan artifacts: {err}",
                style.red("✗ error")
            );
            process::exit(1);
        }
    };
    let run_paths = match prepare_scan_output_run(base_output_root) {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!(
                "  {} failed to create scan output directory: {err}",
                style.red("✗ error")
            );
            process::exit(1);
        }
    };
    let signer = match ScanSigningKey::from_environment() {
        Ok(signer) => signer,
        Err(err) => {
            eprintln!(
                "  {} failed to load DSFB_SCAN_SIGNING_KEY: {err}",
                style.red("✗ error")
            );
            process::exit(1);
        }
    };

    match export_scan_artifacts(report, &run_paths.run_dir, signer.as_ref()) {
        Ok(paths) => {
            eprintln!();
            eprintln!(
                "  {} {}",
                style.green("✔ run dir"),
                style.bold(&paths.output_dir.display().to_string())
            );
            eprintln!(
                "  {} {}",
                style.green("✔ report  "),
                style.dim(&paths.report_path.display().to_string())
            );
            eprintln!(
                "  {} {}",
                style.green("✔ sarif   "),
                style.dim(&paths.sarif_path.display().to_string())
            );
            eprintln!(
                "  {} {}",
                style.green("✔ in-toto "),
                style.dim(&paths.statement_path.display().to_string())
            );
            let dsse_suffix = if paths.signed {
                style.green(" (signed)").to_string()
            } else {
                style
                    .dim(" (unsigned — set DSFB_SCAN_SIGNING_KEY to sign)")
                    .to_string()
            };
            eprintln!(
                "  {} {}{}",
                style.green("✔ dsse    "),
                style.dim(&paths.dsse_path.display().to_string()),
                dsse_suffix
            );
            if let Some(migration_dir) = migrated_legacy_dir {
                eprintln!(
                    "  {} migrated legacy scan artifacts to {}",
                    style.yellow("[note]"),
                    migration_dir.display()
                );
            }
        }
        Err(err) => {
            eprintln!(
                "  {} failed to write scan artifacts: {err}",
                style.red("✗ error")
            );
            process::exit(1);
        }
    }
}

fn print_banner(style: &Style) {
    eprintln!();
    eprintln!(
        "  {}",
        style.cyan("╔═══════════════════════════════════════════════════════════════╗")
    );
    eprintln!(
        "  {}  {}  {}",
        style.cyan("║"),
        style.bold("DSFB-gray · Canonical Broad Audit                          "),
        style.cyan("║")
    );
    eprintln!(
        "  {}",
        style.cyan("╚═══════════════════════════════════════════════════════════════╝")
    );
    eprintln!();
}

struct Style {
    enabled: bool,
}

impl Style {
    fn detect() -> Self {
        let disabled = env::var_os("NO_COLOR").is_some();
        Self {
            enabled: !disabled && io::stderr().is_terminal(),
        }
    }

    fn wrap(&self, code: &str, s: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }

    fn bold(&self, s: &str) -> String {
        self.wrap("1", s)
    }
    fn dim(&self, s: &str) -> String {
        self.wrap("2", s)
    }
    fn cyan(&self, s: &str) -> String {
        self.wrap("36", s)
    }
    fn green(&self, s: &str) -> String {
        self.wrap("32", s)
    }
    fn yellow(&self, s: &str) -> String {
        self.wrap("33", s)
    }
    fn red(&self, s: &str) -> String {
        self.wrap("31", s)
    }
}
