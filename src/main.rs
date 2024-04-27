use librus_export::cli::run_cli;
use anyhow::{Result, Ok};

fn main() -> Result<()> {
    run_cli()?;
    Ok(())
}
