use anyhow::Result;
use dsfb_semiotics_engine::demos::vibration_to_thermal_drift_trace;

fn main() -> Result<()> {
    println!("{}", vibration_to_thermal_drift_trace()?);
    Ok(())
}
