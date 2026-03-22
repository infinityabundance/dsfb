use clap::Parser;

use dsfb_computer_graphics::cli::{run, Cli};
use dsfb_computer_graphics::Result;

fn main() -> Result<()> {
    run(Cli::parse())
}
