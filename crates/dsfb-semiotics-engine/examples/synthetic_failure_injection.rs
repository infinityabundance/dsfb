use anyhow::Result;
use dsfb_semiotics_engine::demos::synthetic_failure_injection_trace;

fn main() -> Result<()> {
    println!("{}", synthetic_failure_injection_trace()?);
    Ok(())
}
