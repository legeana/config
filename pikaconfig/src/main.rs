mod command;
mod empty_struct;
mod file_registry;
mod file_util;
mod git_utils;
mod iter_util;
mod layout;
mod module;
mod package;
mod process_utils;
mod quote;
mod registry;
mod repository;
mod symlink_util;
mod tag_criteria;
mod tag_util;
mod tera_helper;
mod tera_helpers;
mod uninstaller;
mod xdg;
mod xdg_or_win;

use module::{Module, Rules};
use uninstaller::Uninstaller;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const NO_UPDATE_ENV: &str = "PIKACONFIG_NO_UPDATE";
const INSTALL_REGISTRY: &str = ".install";
const SETUP: &str = "setup";

fn config_root(file_in_root: &str) -> Result<PathBuf> {
    let exe_path = env::current_exe()?;
    let mut parent = exe_path.parent();
    while let Some(dir) = parent {
        let file = dir.join(file_in_root);
        if file.exists() {
            return Ok(dir.to_path_buf());
        }
        parent = dir.parent();
    }
    Err(anyhow!("unable to find {file_in_root:?} in project root"))
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
    Update {},
    SystemInstall {
        #[clap(long, default_value_t = true, action = clap::ArgAction::Set)]
        strict: bool,
    },
    Uninstall {},
    ManifestHelp {},
    Tags {},
    List {},
}

fn reload(setup: impl AsRef<Path>) -> Result<()> {
    let setup = setup.as_ref();
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    log::info!("Restarting: $ {setup:?} {args:?}");
    process_utils::run(
        std::process::Command::new(setup)
            .args(args)
            .env(NO_UPDATE_ENV, "yes"),
    )
}

fn registry(root: &Path) -> file_registry::FileRegistry {
    file_registry::FileRegistry::new(root.join(INSTALL_REGISTRY))
}

fn uninstall(root: &Path) -> Result<()> {
    let mut registry = registry(root);
    registry
        .uninstall()
        .context("failed to uninstall before installing")?;
    Ok(())
}

fn install(rules: &Rules, root: &Path) -> Result<()> {
    // Load repositories before uninstalling to abort early.
    // It's better to keep the old configuration than no configuration.
    let repos = layout::repositories(root)?;
    for repo in &repos {
        repo.pre_uninstall(rules)
            .with_context(|| format!("failed to pre-uninstall {}", repo.name()))?;
    }
    let mut registry = registry(root);
    registry
        .uninstall()
        .context("failed to uninstall before installing")?;
    for repo in &repos {
        repo.pre_install(rules, &mut registry)
            .with_context(|| format!("failed to pre-install {}", repo.name()))?;
    }
    for repo in &repos {
        repo.install(rules, &mut registry)
            .with_context(|| format!("failed to install {}", repo.name()))?;
    }
    for repo in &repos {
        repo.post_install(rules, &mut registry)
            .with_context(|| format!("failed to post-install {}", repo.name()))?;
    }
    Ok(())
}

fn system_install(rules: &Rules, root: &Path) -> Result<()> {
    let repos = layout::repositories(root)?;
    for repo in repos.iter() {
        repo.system_install(rules)
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
        .context("failed to initialize stderrlog")?;
    // Main code.
    let root = config_root(SETUP)?;
    let setup = root.join(SETUP);
    log::info!("Found user configuration: {root:?}");
    let check_update = || -> Result<bool> {
        let no_update = args.no_update || env::var(NO_UPDATE_ENV).is_ok();
        if !no_update {
            let need_restart = layout::update(&root)?;
            if need_restart {
                // This process is considered replaced.
                // Don't do anything here.
                reload(setup)?;
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
            let rules = Rules {
                force_download: false,
                ..Rules::default()
            };
            install(&rules, &root).context("failed to install")?;
        }
        Commands::Update {} => {
            if check_update()? {
                return Ok(());
            }
            let rules = Rules {
                force_download: true,
                ..Rules::default()
            };
            install(&rules, &root).context("failed to install")?;
        }
        Commands::SystemInstall { strict } => {
            if check_update()? {
                return Ok(());
            }
            let rules = Rules {
                allow_package_install_failures: !strict,
                ..Rules::default()
            };
            system_install(&rules, &root).context("failed to system_install")?;
        }
        Commands::Uninstall {} => {
            uninstall(&root).context("failed to uninstall")?;
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
                let status = if repo.enabled()? {
                    "[enabled]"
                } else {
                    "[disabled]"
                };
                println!("{} {status}: {}", repo.name(), repo.list().join(", "));
            }
        }
    }
    Ok(())
}
