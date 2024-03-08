use std::env;

use anyhow::Result;

use pikaconfig_bootstrap::cli;
use pikaconfig_bootstrap::dir_layout;
use pikaconfig_bootstrap::logconfig;
use pikaconfig_bootstrap::process_utils;

fn run_pikaconfig() -> Result<()> {
    process_utils::run(
        std::process::Command::new("cargo")
            .arg("run")
            .arg("--release")
            .arg("--package=pikaconfig")
            .arg("--")
            .args(std::env::args().skip(1)), // Skip arg0.
    )
}

fn main() -> Result<()> {
    let args = cli::parse();
    logconfig::init(args.verbose)?;
    // Main code.
    let root = cli::config_root()?;
    let command_needs_update = matches!(
        args.command,
        cli::Commands::Install {} | cli::Commands::Update {} | cli::Commands::SystemInstall {}
    );
    let no_update = args.no_update || env::var(cli::NO_UPDATE_ENV).is_ok();
    if command_needs_update && !no_update {
        // Bootstrap is stable enough we don't attempt to restart.
        // Might reconsider in the future.
        dir_layout::update(&root)?;
    }
    run_pikaconfig()
}
