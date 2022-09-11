mod file_registry;
mod hostname;
mod installer;
mod layout;
mod package;
mod registry;
mod repository;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use log;
use stderrlog;

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const NO_UPDATE_ENV: &str = "PIKACONFIG_NO_UPDATE";
const INSTALL_REGISTRY: &str = ".install";

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
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[clap(short = 'd', long)]
    no_update: bool,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Install {},
    Uninstall {},
    ManifestHelp {},
    List {},
}

fn reload() -> Result<()> {
    let setup = config_root()?.join("setup");
    log::info!(
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

fn registry(root: &Path) -> file_registry::FileRegistry {
    file_registry::FileRegistry::new(root.join(INSTALL_REGISTRY))
}

fn uninstall(root: &Path) -> Result<()> {
    let mut registry = registry(root);
    installer::uninstall(&mut registry)
        .with_context(|| format!("failed to uninstall before installing"))?;
    return Ok(());
}

fn install(root: &Path) -> Result<()> {
    let mut registry = registry(root);
    installer::uninstall(&mut registry)
        .with_context(|| format!("failed to uninstall before installing"))?;
    let repos = layout::repositories(root)?;
    for repo in repos.iter() {
        repo.pre_install_all()
            .with_context(|| format!("failed to pre-install {}", repo.name()))?;
    }
    for repo in repos.iter() {
        repo.install_all(&mut registry)
            .with_context(|| format!("failed to install {}", repo.name()))?;
    }
    for repo in repos.iter() {
        repo.post_install_all()
            .with_context(|| format!("failed to post-install {}", repo.name()))?;
    }
    return Ok(());
}

fn main() -> Result<()> {
    let args = Cli::parse();
    stderrlog::new()
        .timestamp(stderrlog::Timestamp::Off)
        .verbosity(usize::from(args.verbose))
        .init()
        .with_context(|| format!("failed to initialize stderrlog"))?;
    // Main code.
    let root = config_root()?;
    log::info!("Found user configuration: {}", root.display());
    match args.command {
        Commands::Install {} => {
            let no_update = args.no_update || env::var(NO_UPDATE_ENV).is_ok();
            if !no_update {
                let need_restart = layout::update(&root)?;
                if need_restart {
                    // This process is considered replaced.
                    // Don't do anything here.
                    return reload();
                }
            }
            install(&root).with_context(|| format!("failed to install"))?;
        }
        Commands::Uninstall {} => {
            uninstall(&root).with_context(|| format!("failed to uninstall"))?;
        }
        Commands::ManifestHelp {} => {
            print!("{}", package::manifest_help());
        }
        Commands::List {} => {
            for repos in layout::repositories(&root).iter() {
                for repo in repos.iter() {
                    println!("{}: {}", repo.name(), repo.list().join(", "));
                }
            }
        }
    }
    return Ok(());
}
