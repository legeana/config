mod config;
mod contents;
mod installer;
mod satisficer;
mod system;
mod user;

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, anyhow};
use registry::Registry;

use crate::module::{self, Module, ModuleBox, Rules};
use crate::package::installer::Installer;
use crate::tag_criteria::{self, Criteria as _};

pub use contents::help as manifest_help;

pub struct Package {
    name: String,
    criteria: Option<tag_criteria::TagCriteria>,
    modules: Vec<ModuleBox>,
    #[allow(dead_code)]
    dependencies: Vec<String>,
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
    for dep in dependencies {
        if !dep
            .requires
            .is_satisfied()
            .context("failed to check tags")?
        {
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
        let criteria = pkgconfig.requires.clone();
        let backup_name = name_from_path(&root)?;
        let dependencies: Vec<String> = match pkgconfig.dependencies {
            Some(deps) => filter_dependencies(&deps)
                .with_context(|| format!("failed to get dependencies of {root:?}"))?,
            None => Vec::new(),
        };
        let system_dependencies: Vec<system::SystemDependency> = pkgconfig
            .system_dependencies
            .unwrap_or_default()
            .iter()
            .map(system::SystemDependency::new)
            .collect::<Result<_>>()
            .context("failed to parse system_dependencies")?;
        let user_dependencies: Vec<user::UserDependency> = pkgconfig
            .user_dependencies
            .unwrap_or_default()
            .iter()
            .map(user::UserDependency::new)
            .collect::<Result<_>>()
            .context("failed to parse user_dependencies")?;
        let configuration: ModuleBox = if pkgconfig.has_contents {
            if criteria.is_satisfied()? {
                contents::new(root)?
            } else {
                contents::verify(&root)?;
                module::dummy_box()
            }
        } else {
            module::dummy_box()
        };
        Ok(Self {
            name: pkgconfig.name.unwrap_or(backup_name),
            criteria,
            modules: vec![
                configuration,
                module::wrap_keep_going(system_dependencies),
                module::wrap_user_deps(module::wrap_keep_going(user_dependencies)),
            ],
            dependencies,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    fn enabled(&self) -> Result<bool> {
        self.criteria.is_satisfied()
    }
    fn run_if_enabled<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        if !self
            .enabled()
            .with_context(|| format!("failed to check if {} is enabled", self.name()))?
        {
            log::info!("Skipping disabled {}", self.name());
            log::debug!("{} is disabled: {:?}", self.name(), self.criteria);
            return Ok(());
        }
        f()
    }
}

impl Module for Package {
    fn pre_uninstall(&self, rules: &Rules) -> Result<()> {
        self.run_if_enabled(|| self.modules.pre_uninstall(rules))
            .with_context(|| format!("{}: failed pre_uninstall", self.name()))
    }
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.run_if_enabled(|| self.modules.pre_install(rules, registry))
            .with_context(|| format!("{}: failed pre_install", self.name()))
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.run_if_enabled(|| self.modules.install(rules, registry))
            .with_context(|| format!("{}: failed install", self.name()))
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.run_if_enabled(|| self.modules.post_install(rules, registry))
            .with_context(|| format!("{}: failed post_install", self.name()))
    }
    fn system_install(&self, rules: &Rules) -> Result<()> {
        self.run_if_enabled(|| self.modules.system_install(rules))
            .with_context(|| format!("{}: failed system_install", self.name()))
    }
}
