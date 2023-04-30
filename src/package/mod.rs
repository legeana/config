mod config;
mod contents;
mod system;
mod user;

use std::path::{Path, PathBuf};

use crate::registry::Registry;
use crate::tag_criteria;

use anyhow::{anyhow, Context, Result};

pub use contents::help as manifest_help;

trait Installer {
    fn install(&self) -> Result<()>;
}

impl<T: Installer> Installer for Vec<T> {
    fn install(&self) -> Result<()> {
        for installer in self.iter() {
            installer.install()?;
        }
        Ok(())
    }
}

pub struct Package {
    name: String,
    criteria: tag_criteria::Criteria,
    configuration: contents::Configuration,
    #[allow(dead_code)]
    dependencies: Vec<String>,
    system_dependency: Vec<system::SystemDependency>,
    user_dependency: Vec<user::UserDependency>,
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
        if !tag_criteria::is_satisfied(dep).context("failed to check tags")? {
            continue;
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
            Some(deps) => deps
                .iter()
                .map(system::SystemDependency::new)
                .collect::<Result<_>>()
                .context("failed to parse system_dependencies")?,
            None => Vec::<system::SystemDependency>::default(),
        };
        let user_dependency = match pkgconfig.user_dependencies {
            Some(deps) => deps
                .iter()
                .map(user::UserDependency::new)
                .collect::<Result<_>>()
                .context("failed to parse user_dependencies")?,
            None => Vec::<user::UserDependency>::default(),
        };
        let configuration = if pkgconfig.has_contents {
            contents::Configuration::new(root)?
        } else {
            contents::Configuration::new_empty(root)
        };
        Ok(Package {
            name: pkgconfig.name.unwrap_or(backup_name),
            criteria: tag_criteria::Criteria {
                requires: pkgconfig.requires,
                conflicts: pkgconfig.conflicts,
            },
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
        self.criteria.is_satisfied()
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
