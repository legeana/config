#![allow(
    // This lint is too opinionated.
    // In situations where enum name matches outside class
    // the consistency is more important than repetition.
    clippy::enum_variant_names,
)]

mod annotated_path;
mod command;
mod empty_struct;
mod file_registry;
mod file_util;
mod iter_util;
mod layout;
mod module;
mod package;
mod quote;
mod registry;
mod repository;
mod symlink_util;
mod tag_criteria;
mod tag_util;
mod tera_helper;
mod tera_helpers;
mod unarchiver;
mod uninstaller;
mod xdg;
mod xdg_or_win;

use std::path::Path;

use anyhow::{Context, Result};

// Pretend these modules are local.
use pikaconfig_bootstrap::{cli, dir_layout, git_utils, logconfig, process_utils, shlexfmt};

use module::{Module, Rules};
use uninstaller::Uninstaller;

const INSTALL_REGISTRY: &str = ".install";
const STATE_REGISTRY: &str = ".state";

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
    let args = cli::parse();
    logconfig::init(args.verbose)?;
    // Main code.
    let root = cli::config_root()?;
    log::info!("Found user configuration: {root:?}");
    match args.command {
        cli::Commands::Install {} => {
            let rules = Rules {
                force_update: false,
                force_reinstall: false,
                keep_going: args.keep_going,
                user_deps: !args.no_user_deps,
            };
            install(&rules, &root).context("failed to install")?;
        }
        cli::Commands::Update {} => {
            let rules = Rules {
                force_update: true,
                force_reinstall: false,
                keep_going: args.keep_going,
                user_deps: !args.no_user_deps,
            };
            install(&rules, &root).context("failed to install")?;
        }
        cli::Commands::Reinstall {} => {
            let rules = Rules {
                force_update: true,
                force_reinstall: true,
                keep_going: args.keep_going,
                user_deps: !args.no_user_deps,
            };
            install(&rules, &root).context("failed to install")?;
        }
        cli::Commands::SystemInstall {} => {
            let rules = Rules {
                keep_going: args.keep_going,
                ..Default::default()
            };
            system_install(&rules, &root).context("failed to system_install")?;
        }
        cli::Commands::Uninstall {} => {
            uninstall(&root).context("failed to uninstall")?;
        }
        cli::Commands::ManifestHelp {} => {
            print!("{}", package::manifest_help());
        }
        cli::Commands::Tags {} => {
            for tag in tag_util::tags().context("failed to get tags")? {
                println!("{}", tag);
            }
        }
        cli::Commands::List {} => {
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
