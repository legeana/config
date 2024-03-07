use anyhow::{bail, Result};

use pikaconfig_bootstrap::cli;
use pikaconfig_bootstrap::logconfig;

fn main() -> Result<()> {
    let args = cli::parse();
    logconfig::init(args.verbose)?;
    // Main code.
    bail!("Not implemented yet");
}
