use dsfb_gray::{build_public_evaluation, write_public_artifacts};
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let mut args = env::args_os();
    let _program = args.next();
    let output_root = match args.next() {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("."),
    };
    if args.next().is_some() {
        eprintln!("usage: cargo run --bin dsfb-regenerate-public-artifacts -- [OUTPUT_ROOT]");
        process::exit(2);
    }

    let bundle = build_public_evaluation();
    match write_public_artifacts(&bundle, &output_root) {
        Ok(paths) => {
            eprintln!(
                "wrote {}, {}, {}, {}, {}",
                paths.evaluation_results_path.display(),
                paths.demo_output_path.display(),
                paths.sensitivity_sweep_path.display(),
                paths.generated_docs_dir.display(),
                paths.generated_paper_dir.display()
            );
        }
        Err(err) => {
            eprintln!("error: failed to regenerate public artifacts: {err}");
            process::exit(1);
        }
    }
}
