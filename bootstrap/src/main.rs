use anyhow::Result;

use pikaconfig_bootstrap::cli;
use pikaconfig_bootstrap::dir_layout;
use pikaconfig_bootstrap::logconfig;

fn main() -> Result<()> {
    let args = cli::parse();
    logconfig::init(args.verbose)?;
    // Main code.
    let root = cli::config_root()?;
    let command_needs_update = matches!(
        args.command,
        cli::Commands::Install {} | cli::Commands::Update {} | cli::Commands::SystemInstall {}
    );
    if command_needs_update && !args.no_update {
        // Bootstrap is stable enough we don't attempt to restart.
        // Might reconsider in the future.
        dir_layout::update(&root)?;
    }
    log::debug!("Bootstrapped successfully");
    Ok(())
}
