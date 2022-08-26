mod layout;
mod package;
mod repository;

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};

use std::env;
use std::path::{Path, PathBuf};

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
            if !args.no_update {
                layout::update(&root)?;
            }
            debug(&root)?;
        }
        Commands::Uninstall {} => {
            println!("uninstalling");
        }
    }
    return Ok(());
}
