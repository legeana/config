mod file_registry;
mod layout;
mod package;
mod registry;
mod repository;
mod tag_util;
mod uninstaller;

use uninstaller::Uninstaller;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

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
    SystemInstall {
        #[clap(long, default_value_t = true, action = clap::ArgAction::Set)]
        strict: bool,
    },
    Uninstall {},
    ManifestHelp {},
    Tags {},
    List {},
}

fn reload() -> Result<()> {
    let setup = config_root()?.join("setup");
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    log::info!("Restarting: $ {setup:?} {args:?}");
    let exit_status = std::process::Command::new(&setup)
        .args(args)
        .env(NO_UPDATE_ENV, "yes")
        .status()?;
    if !exit_status.success() {
        return Err(anyhow!("failed to run {setup:?}"));
    }
    Ok(())
}

fn registry(root: &Path) -> file_registry::FileRegistry {
    file_registry::FileRegistry::new(root.join(INSTALL_REGISTRY))
}

fn uninstall(root: &Path) -> Result<()> {
    let mut registry = registry(root);
    registry
        .uninstall()
        .with_context(|| "failed to uninstall before installing")?;
    Ok(())
}

fn install(root: &Path) -> Result<()> {
    // Load repositories before uninstalling to abort early.
    // It's better to keep the old configuration than no configuration.
    let repos = layout::repositories(root)?;
    let mut registry = registry(root);
    registry
        .uninstall()
        .with_context(|| "failed to uninstall before installing")?;
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
    Ok(())
}

fn system_install(root: &Path, strict: bool) -> Result<()> {
    let repos = layout::repositories(root)?;
    for repo in repos.iter() {
        repo.system_install_all(strict)
            .with_context(|| format!("failed to system_install {}", repo.name()))?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    stderrlog::new()
        .timestamp(stderrlog::Timestamp::Off)
        .verbosity(usize::from(args.verbose))
        .init()
        .with_context(|| "failed to initialize stderrlog")?;
    // Main code.
    let root = config_root()?;
    log::info!("Found user configuration: {root:?}");
    let check_update = || -> Result<bool> {
        let no_update = args.no_update || env::var(NO_UPDATE_ENV).is_ok();
        if !no_update {
            let need_restart = layout::update(&root)?;
            if need_restart {
                // This process is considered replaced.
                // Don't do anything here.
                reload()?;
                return Ok(true);
            }
        }
        Ok(false)
    };
    match args.command {
        Commands::Install {} => {
            if check_update()? {
                return Ok(());
            }
            install(&root).with_context(|| "failed to install")?;
        }
        Commands::SystemInstall { strict } => {
            if check_update()? {
                return Ok(());
            }
            system_install(&root, strict).with_context(|| "failed to system_install")?;
        }
        Commands::Uninstall {} => {
            uninstall(&root).with_context(|| "failed to uninstall")?;
        }
        Commands::ManifestHelp {} => {
            print!("{}", package::manifest_help());
        }
        Commands::Tags {} => {
            for tag in tag_util::tags().context("failed to get tags")? {
                println!("{}", tag);
            }
        }
        Commands::List {} => {
            let repos = layout::repositories(&root)
                .with_context(|| format!("failed to get repositories from {root:?}"))?;
            for repo in repos.iter() {
                let status = if repo.enabled()? { "[enabled]" } else { "[disabled]" };
                println!("{} {status}: {}", repo.name(), repo.list().join(", "));
            }
        }
    }
    Ok(())
}
