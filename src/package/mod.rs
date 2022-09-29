mod config;
mod contents;
mod system;

use std::path::{Path, PathBuf};

use crate::registry::Registry;
use crate::tag_util;

use anyhow::{anyhow, Context, Result};

pub use contents::help as manifest_help;

pub struct Package {
    name: String,
    configuration: contents::Configuration,
    #[allow(dead_code)]
    dependencies: Vec<String>,
    system_dependency: system::SystemDependencyGroup,
    user_dependency: system::UserDependencyGroup,
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
        let pkgconfig_result = config::load_package(&root);
        match pkgconfig_result {
            Err(err) => match err {
                config::Error::FileNotFound(_) => Self::new_no_pkg(root),
                config::Error::Other(err) => {
                    Err(err.context(format!("failed to load package config for {root:?}")))
                }
            },
            Ok(pkgconfig) => Self::new_with_pkg(root, pkgconfig),
        }
    }
    fn new_with_pkg(root: PathBuf, pkgconfig: config::Package) -> Result<Self> {
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
            Some(deps) => system::UserDependencyGroup::new(&deps)
                .context("failed to parse user_dependencies")?,
            None => system::UserDependencyGroup::default(),
        };
        Ok(Package {
            name: pkgconfig.name.unwrap_or(backup_name),
            configuration: contents::Configuration::new(root)?,
            dependencies,
            system_dependency,
            user_dependency,
        })
    }
    fn new_no_pkg(root: PathBuf) -> Result<Self> {
        Ok(Package {
            name: name_from_path(&root)?,
            configuration: contents::Configuration::new(root)?,
            dependencies: Vec::new(),
            system_dependency: system::SystemDependencyGroup::default(),
            user_dependency: system::UserDependencyGroup::default(),
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn pre_install(&self) -> Result<()> {
        self.configuration.pre_install()
    }
    pub fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        self.configuration.install(registry)
    }
    pub fn post_install(&self) -> Result<()> {
        self.configuration.post_install()
    }
    pub fn system_install(&self) -> Result<()> {
        self.system_dependency
            .install()
            .context("failed to install system dependencies")?;
        self.user_dependency
            .install()
            .context("failed to install user dependencies")
    }
}
