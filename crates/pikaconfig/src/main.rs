use std::path::Path;

use anyhow::{Context, Result};

use pikaconfig::layout;
use pikaconfig::module::{Module, Rules};
use pikaconfig::package;
use pikaconfig::tag_util;
use pikaconfig::uninstaller::Uninstaller;

const SQL_REGISTRY: &str = ".install.sqlite";

fn registry(root: &Path) -> Result<registry::sqlite::SqliteRegistry> {
    let sql_path = root.join(SQL_REGISTRY);
    registry::sqlite::SqliteRegistry::open(&sql_path)
        .with_context(|| format!("failed to open SQLite registry {sql_path:?}"))
}

fn uninstall(root: &Path) -> Result<()> {
    let mut u = registry(root)?;
    u.uninstall().context("failed to uninstall user files")?;
    u.cleanup().context("failed to cleanup state")?;
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
    let mut registry = registry(root)?;
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
    registry.close().context("failed to close SqliteRegistry")
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
    cli::logconfig::init(args.quiet, args.verbose)?;
    // Main code.
    let root = cli::config_root()?;
    log::info!("Found user configuration: {root:?}");
    match args.command {
        cli::Commands::Install => {
            let rules = Rules {
                force_update: false,
                force_reinstall: false,
                keep_going: args.keep_going,
                user_deps: !args.no_user_deps,
            };
            install(&rules, &root).context("failed to install")?;
        }
        cli::Commands::Update => {
            let rules = Rules {
                force_update: true,
                force_reinstall: false,
                keep_going: args.keep_going,
                user_deps: !args.no_user_deps,
            };
            install(&rules, &root).context("failed to install")?;
        }
        cli::Commands::Reinstall => {
            let rules = Rules {
                force_update: true,
                force_reinstall: true,
                keep_going: args.keep_going,
                user_deps: !args.no_user_deps,
            };
            install(&rules, &root).context("failed to install")?;
        }
        cli::Commands::SystemInstall => {
            let rules = Rules {
                keep_going: args.keep_going,
                ..Default::default()
            };
            system_install(&rules, &root).context("failed to system_install")?;
        }
        cli::Commands::Uninstall => {
            uninstall(&root).context("failed to uninstall")?;
        }
        cli::Commands::ManifestHelp => {
            print!("{}", package::manifest_help());
        }
        cli::Commands::Tags => {
            for tag in tag_util::tags().context("failed to get tags")? {
                println!("{}", tag);
            }
        }
        cli::Commands::List => {
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
        cli::Commands::MigrateRegistry => {
            registry(&root).context("failed to load registry")?;
        }
    }
    Ok(())
}
