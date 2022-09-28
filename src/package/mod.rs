mod config;
mod contents;
mod system;

use std::path::{Path, PathBuf};

use crate::registry::Registry;

use anyhow::{anyhow, Context, Result};

pub use contents::help as manifest_help;

pub struct Package {
    name: String,
    configuration: contents::Configuration,
    #[allow(dead_code)]
    dependencies: Vec<String>,
    system_dependency: system::SystemDependencyGroup,
}

fn name_from_path(path: &Path) -> Result<String> {
    Ok(path
        .file_name()
        .ok_or_else(|| anyhow!("failed to get {path:?} basename"))?
        .to_string_lossy()
        .into())
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
        let dependencies: Vec<String> = pkgconfig
            .dependencies
            .unwrap_or_default()
            .iter()
            .map(|dep| dep.name.clone())
            .collect();
        let system_dependency = match pkgconfig.system_dependencies {
            Some(deps) => system::SystemDependencyGroup::new(&deps)
                .context("failed to parse system_dependencies")?,
            None => system::SystemDependencyGroup::default(),
        };
        Ok(Package {
            name: pkgconfig.name.unwrap_or(backup_name),
            configuration: contents::Configuration::new(root)?,
            dependencies,
            system_dependency,
        })
    }
    fn new_no_pkg(root: PathBuf) -> Result<Self> {
        Ok(Package {
            name: name_from_path(&root)?,
            configuration: contents::Configuration::new(root)?,
            dependencies: Vec::new(),
            system_dependency: system::SystemDependencyGroup::default(),
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
        self.system_dependency.install()
    }
}
