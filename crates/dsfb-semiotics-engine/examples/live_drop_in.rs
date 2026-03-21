use anyhow::Result;
use dsfb_semiotics_engine::demos::live_drop_in_trace;

fn main() -> Result<()> {
    println!("{}", live_drop_in_trace()?);
    Ok(())
}
