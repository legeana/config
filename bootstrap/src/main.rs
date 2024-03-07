use anyhow::Result;

use pikaconfig_bootstrap::cli;
use pikaconfig_bootstrap::logconfig;
use pikaconfig_bootstrap::process_utils;

fn run_pikaconfig() -> Result<()> {
    process_utils::run(
        std::process::Command::new("cargo")
            .arg("run")
            .arg("--package=pikaconfig")
            .arg("--")
            .args(std::env::args().skip(1)), // Skip arg0.
    )
}

fn main() -> Result<()> {
    let args = cli::parse();
    logconfig::init(args.verbose)?;
    // Main code.
    run_pikaconfig()
}
