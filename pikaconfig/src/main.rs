#![allow(
    // This lint is too opinionated.
    // In situations where enum name matches outside class
    // the consistency is more important than repetition.
    clippy::enum_variant_names,
)]

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
mod shlexfmt;
mod symlink_util;
mod tag_criteria;
mod tag_util;
mod tera_helper;
mod tera_helpers;
mod unarchiver;
mod uninstaller;
mod xdg;
mod xdg_or_win;

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use module::{Module, Rules};
use uninstaller::Uninstaller;

const NO_UPDATE_ENV: &str = "PIKACONFIG_NO_UPDATE";
const CONFIG_ROOT_ENV: &str = "PIKACONFIG_CONFIG_ROOT";
const INSTALL_REGISTRY: &str = ".install";
const STATE_REGISTRY: &str = ".state";

fn config_root() -> Result<PathBuf> {
    let config_root = env::var_os(CONFIG_ROOT_ENV)
        .with_context(|| format!("failed to read {CONFIG_ROOT_ENV}, use setup"))?;
    Ok(config_root.into())
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[clap(short = 'd', long)]
    no_update: bool,
    #[clap(subcommand)]
    command: Commands,
    #[clap(
        short = 'k',
        long,
        help = "Don't interrupt installation process if a package fails"
    )]
    keep_going: bool,
    #[clap(long, help = "Don't install user dependencies")]
    no_user_deps: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Install {},
    Update {},
    SystemInstall {},
    Uninstall {},
    ManifestHelp {},
    Tags {},
    List {},
}

fn registry(root: &Path) -> file_registry::FileRegistry {
    let user_files_path = root.join(INSTALL_REGISTRY);
    let state_files_path = root.join(STATE_REGISTRY);
    file_registry::FileRegistry::new(user_files_path, state_files_path)
}

fn uninstall(root: &Path) -> Result<()> {
    let mut registry = registry(root);
    registry
        .uninstall()
        .context("failed to uninstall user files")?;
    registry.cleanup().context("failed to cleanup state")?;
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
    env_logger::Builder::new()
        .filter_level(match args.verbose {
            0 => log::LevelFilter::Off,
            1 => log::LevelFilter::Error,
            2 => log::LevelFilter::Warn,
            3 => log::LevelFilter::Info,
            4 => log::LevelFilter::Debug,
            5 => log::LevelFilter::Trace,
            6.. => bail!("invalid log level: {}", args.verbose),
        })
        .default_format()
        .format_timestamp(None)
        .format_target(false)
        .try_init()?;
    // Main code.
    let root = config_root()?;
    log::info!("Found user configuration: {root:?}");
    let check_update = || -> Result<()> {
        let no_update = args.no_update || env::var(NO_UPDATE_ENV).is_ok();
        if !no_update {
            layout::update(&root)?;
        }
        Ok(())
    };
    match args.command {
        Commands::Install {} => {
            check_update()?;
            let rules = Rules {
                force_download: false,
                keep_going: args.keep_going,
                user_deps: !args.no_user_deps,
            };
            install(&rules, &root).context("failed to install")?;
        }
        Commands::Update {} => {
            check_update()?;
            let rules = Rules {
                force_download: true,
                keep_going: args.keep_going,
                user_deps: !args.no_user_deps,
            };
            install(&rules, &root).context("failed to install")?;
        }
        Commands::SystemInstall {} => {
            check_update()?;
            let rules = Rules {
                keep_going: args.keep_going,
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
