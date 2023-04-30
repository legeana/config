mod config;
mod contents;
mod system;
mod user;

use std::path::{Path, PathBuf};

use crate::registry::Registry;
use crate::tag_util;

use anyhow::{anyhow, Context, Result};

pub use contents::help as manifest_help;

trait Installer {
    fn install(&self) -> Result<()>;
}

pub struct Package {
    name: String,
    requires: Vec<String>,
    conflicts: Vec<String>,
    configuration: contents::Configuration,
    #[allow(dead_code)]
    dependencies: Vec<String>,
    system_dependency: system::SystemDependencyGroup,
    user_dependency: user::UserDependencyGroup,
}

fn name_from_path(path: &Path) -> Result<String> {
    Ok(path
        .file_name()
        .ok_or_else(|| anyhow!("failed to get {path:?} basename"))?
        .to_string_lossy()
        .into())
}

fn filter_dependencies(dependencies: &[config::Dependency]) -> Result<Vec<String>> {
    let mut deps: Vec<String> = Vec::new();
    for dep in dependencies.iter() {
        if let Some(tags) = &dep.requires {
            if !tag_util::has_all_tags(tags).context("failed has_all_tags")? {
                continue;
            }
        }
        if let Some(tags) = &dep.conflicts {
            if tag_util::has_any_tags(tags).context("failed has_any_tags")? {
                continue;
            }
        }
        deps.extend(dep.names.iter().cloned());
    }
    Ok(deps)
}

impl Package {
    pub fn new(root: PathBuf) -> Result<Self> {
        let pkgconfig = config::load_package(&root)
            .with_context(|| format!("failed to load {root:?} package"))?;
        let backup_name = name_from_path(&root)?;
        let dependencies: Vec<String> = match pkgconfig.dependencies {
            Some(deps) => filter_dependencies(&deps)
                .with_context(|| format!("failed to get dependencies of {root:?}"))?,
            None => Vec::new(),
        };
        let system_dependency = match pkgconfig.system_dependencies {
            Some(deps) => system::SystemDependencyGroup::new(&deps)
                .context("failed to parse system_dependencies")?,
            None => system::SystemDependencyGroup::default(),
        };
        let user_dependency = match pkgconfig.user_dependencies {
            Some(deps) => user::UserDependencyGroup::new(&deps)
                .context("failed to parse user_dependencies")?,
            None => user::UserDependencyGroup::default(),
        };
        let configuration = if pkgconfig.has_contents {
            contents::Configuration::new(root)?
        } else {
            contents::Configuration::new_empty(root)
        };
        Ok(Package {
            name: pkgconfig.name.unwrap_or(backup_name),
            requires: pkgconfig.requires.unwrap_or_default(),
            conflicts: pkgconfig.conflicts.unwrap_or_default(),
            configuration,
            dependencies,
            system_dependency,
            user_dependency,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    fn enabled(&self) -> Result<bool> {
        if !tag_util::has_all_tags(&self.requires).context("failed has_all_tags")? {
            return Ok(false);
        }
        if tag_util::has_any_tags(&self.conflicts).context("failed has_any_tags")? {
            return Ok(false);
        }
        Ok(true)
    }
    pub fn pre_install(&self) -> Result<()> {
        if !self
            .enabled()
            .with_context(|| format!("failed to check if {} is enabled", self.name()))?
        {
            log::info!("Skipping disabled {}", self.name());
            return Ok(());
        }
        self.user_dependency
            .install()
            .context("failed to install user dependencies")?;
        self.configuration.pre_install()
    }
    pub fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        if !self
            .enabled()
            .with_context(|| format!("failed to check if {} is enabled", self.name()))?
        {
            log::info!("Skipping disabled {}", self.name());
            return Ok(());
        }
        self.configuration.install(registry)
    }
    pub fn post_install(&self) -> Result<()> {
        if !self
            .enabled()
            .with_context(|| format!("failed to check if {} is enabled", self.name()))?
        {
            log::info!("Skipping disabled {}", self.name());
            return Ok(());
        }
        self.configuration.post_install()
    }
    pub fn system_install(&self) -> Result<()> {
        if !self
            .enabled()
            .with_context(|| format!("failed to check if {} is enabled", self.name()))?
        {
            log::info!("Skipping disabled {}", self.name());
            return Ok(());
        }
        self.system_dependency
            .install()
            .context("failed to install system dependencies")
    }
}
