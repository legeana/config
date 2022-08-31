mod layout;
mod package;
mod registry;
mod repository;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const NO_UPDATE_ENV: &str = "PIKACONFIG_NO_UPDATE";

fn config_root() -> Result<PathBuf> {
    let exe_path = env::current_exe()?;
    let mut parent = exe_path.parent();
    while let Some(dir) = parent {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(dir.to_path_buf());
        }
        parent = dir.parent();
    }
    Err(anyhow!("unable to find Cargo.toml, use setup helper"))
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long)]
    verbose: bool,
    #[clap(short = 'd', long)]
    no_update: bool,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Install {},
    Uninstall {},
}

fn reload() -> Result<()> {
    let setup = config_root()?.join("setup");
    println!(
        "Restarting: $ {} {:?}",
        setup.display(),
        env::args_os().skip(1).collect::<Vec<OsString>>()
    );
    let exit_status = std::process::Command::new(&setup)
        .args(env::args_os().skip(1))
        .env(NO_UPDATE_ENV, "yes")
        .status()?;
    if !exit_status.success() {
        return Err(anyhow!("failed to run {}", setup.display()));
    }
    return Ok(());
}

fn debug(root: &Path) -> Result<()> {
    let repos = layout::repositories(root)?;
    for repo in repos {
        println!("{}: {:?}", repo.name(), repo.list());
    }
    return Ok(());
}

fn main() -> Result<()> {
    let root = config_root()?;
    println!("Found user configuration: {}", root.display());
    let args = Cli::parse();
    match args.command {
        Commands::Install {} => {
            let no_update = args.no_update
                || match env::var(NO_UPDATE_ENV) {
                    Ok(_) => true,
                    Err(_) => false,
                };
            if !no_update {
                let need_restart = layout::update(&root)?;
                if need_restart {
                    // This process is considered replaced.
                    // Don't do anything here.
                    return reload();
                }
            }
            debug(&root)?;
        }
        Commands::Uninstall {} => {
            println!("uninstalling");
        }
    }
    return Ok(());
}
