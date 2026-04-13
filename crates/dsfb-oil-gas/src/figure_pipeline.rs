use crate::figure_traces::export_grammar_traces;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{boxed::Box, format, println, string::String, vec, vec::Vec};

pub struct FigureGenerationResult {
    pub output_dir: PathBuf,
    pub figure_count: usize,
    pub booklet_generated: bool,
    pub archive_generated: bool,
}

pub fn generate_all_figures() -> Result<FigureGenerationResult, Box<dyn std::error::Error>> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_dir = create_fresh_output_dir(&crate_root)?;
    let figures_dir = output_dir.join("figures");

    println!("═══════════════════════════════════════════════════════");
    println!("  DSFB Oil & Gas — Figure Pipeline");
    println!("  Crate  : {}", crate_root.display());
    println!("  Output : {}", output_dir.display());
    println!("═══════════════════════════════════════════════════════");

    println!();
    println!("► Step 1/4: Rust export (shared crate path)");
    let export = export_grammar_traces(&crate_root)?;
    println!("  Petrobras 3W  : {} steps", export.steps_3w);
    println!("  Equinor Volve : {} steps", export.steps_volve);
    println!("  RPDBCS ESP    : {} steps", export.steps_esp);
    println!("  Trace CSVs    : {}", export.trace_dir.display());

    println!();
    println!("► Step 2/4: Figure generation (python3 scripts/gen_figures.py)");
    run_required_command(
        "python3",
        ["scripts/gen_figures.py"],
        &crate_root,
        Some(("DSFB_OUTPUT", output_dir.as_os_str())),
    )?;

    println!();
    println!("► Step 3/4: LaTeX figure booklet (all_figures.tex)");
    fs::copy(
        crate_root.join("figures").join("all_figures.tex"),
        figures_dir.join("all_figures.tex"),
    )?;
    let booklet_generated = compile_booklet(&figures_dir)?;

    println!();
    println!("► Step 4/4: Creating zip archive");
    let archive_generated = create_archive(&figures_dir)?;

    let figure_count = count_individual_figures(&figures_dir)?;
    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("  DONE");
    println!(
        "  Individual figures : {}  ({}/fig_*.pdf)",
        figure_count,
        figures_dir.display()
    );
    if booklet_generated {
        println!("  Figure booklet    : {}", figures_dir.join("all_figures.pdf").display());
    }
    if archive_generated {
        println!("  Download archive  : {}", figures_dir.join("dsfb_figures.zip").display());
    }
    println!("═══════════════════════════════════════════════════════");

    Ok(FigureGenerationResult {
        output_dir,
        figure_count,
        booklet_generated,
        archive_generated,
    })
}

fn create_fresh_output_dir(crate_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let workspace_root = crate_root
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("failed to resolve workspace root"))?;
    let output_root = workspace_root.join("output-dsfb-oil-gas");
    fs::create_dir_all(&output_root)?;

    let stamp = current_timestamp()?;
    let base_name = format!("dsfb-oil-gas-{stamp}");
    let mut candidate = output_root.join(&base_name);
    let mut suffix = 1usize;
    while candidate.exists() {
        candidate = output_root.join(format!("{base_name}-{suffix:02}"));
        suffix += 1;
    }

    fs::create_dir_all(candidate.join("figures"))?;
    Ok(candidate)
}

fn current_timestamp() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("date")
        .arg("+%Y-%m-%d-%H%M%S")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("date command failed").into());
    }

    let stamp = String::from_utf8(output.stdout)?;
    Ok(String::from(stamp.trim()))
}

fn run_required_command<I, S>(
    program: &str,
    args: I,
    cwd: &Path,
    env: Option<(&str, &OsStr)>,
) -> Result<(), Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args).current_dir(cwd);
    if let Some((key, value)) = env {
        command.env(key, value);
    }

    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{program} exited with status {status}")).into())
    }
}

fn compile_booklet(figures_dir: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let tex_name = "all_figures.tex";
    match run_optional_command(
        "pdflatex",
        ["-interaction=nonstopmode", tex_name],
        figures_dir,
        None,
    )? {
        Some(()) => {
            run_required_command(
                "pdflatex",
                ["-interaction=nonstopmode", tex_name],
                figures_dir,
                None,
            )?;
            println!("  all_figures.pdf compiled");
            Ok(true)
        }
        None => {
            println!("  WARNING: pdflatex not found — skipping booklet");
            Ok(false)
        }
    }
}

fn create_archive(figures_dir: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let mut files: Vec<String> = fs::read_dir(figures_dir)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("fig_") && name.ends_with(".pdf") {
                Some(name.into_owned())
            } else {
                None
            }
        })
        .collect();
    files.sort();
    if figures_dir.join("all_figures.pdf").exists() {
        files.push(String::from("all_figures.pdf"));
    }
    if files.is_empty() {
        println!("  WARNING: no PDFs found — skipping archive");
        return Ok(false);
    }

    let mut args = vec![String::from("-q"), String::from("dsfb_figures.zip")];
    args.extend(files);
    match run_optional_command("zip", args.iter().map(String::as_str), figures_dir, None)? {
        Some(()) => Ok(true),
        None => {
            println!("  WARNING: zip not found — skipping archive");
            Ok(false)
        }
    }
}

fn run_optional_command<I, S>(
    program: &str,
    args: I,
    cwd: &Path,
    env: Option<(&str, &OsStr)>,
) -> Result<Option<()>, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args).current_dir(cwd);
    if let Some((key, value)) = env {
        command.env(key, value);
    }

    match command.status() {
        Ok(status) if status.success() => Ok(Some(())),
        Ok(status) => Err(io::Error::other(format!("{program} exited with status {status}")).into()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn count_individual_figures(figures_dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    Ok(fs::read_dir(figures_dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.starts_with("fig_") && name.ends_with(".pdf")
        })
        .count())
}
