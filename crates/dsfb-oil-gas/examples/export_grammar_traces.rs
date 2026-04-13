use dsfb_oil_gas::export_grammar_traces;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let export = export_grammar_traces(&crate_root)?;

    println!();
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║  DSFB GRAMMAR TRACE EXPORT — REAL DATA ONLY            ║");
    println!("╠════════════════════════════════════════════════════════╣");
    println!("║  Petrobras 3W    →  real_3w_trace.csv    {:>6} steps  ║", export.steps_3w);
    println!("║  Equinor Volve   →  real_volve_trace.csv {:>6} steps  ║", export.steps_volve);
    println!("║  RPDBCS ESP      →  real_esp_trace.csv   {:>6} steps  ║", export.steps_esp);
    println!("╠════════════════════════════════════════════════════════╣");
    println!("║  Output: {:<43} ║", export.trace_dir.display());
    println!("╚════════════════════════════════════════════════════════╝");
    Ok(())
}
