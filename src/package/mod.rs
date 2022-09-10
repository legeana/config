mod configuration;
mod system;

use std::path::PathBuf;

use crate::registry::Registry;
use configuration::Configuration;

use anyhow::{anyhow, Result};

pub use configuration::help as manifest_help;

pub struct Package {
    name: String,
    configuration: Configuration,
    dependencies: Vec<String>,
    system_dependencies: Vec<system::SystemPackage>,
}

impl Package {
    pub fn new(root: PathBuf) -> Result<Self> {
        let name: String = root
            .file_name()
            .ok_or(anyhow!("failed to get {} basename", root.display()))?
            .to_string_lossy()
            .into();
        Ok(Package {
            name,
            configuration: Configuration::new(root)?,
            dependencies: Vec::new(),
            system_dependencies: Vec::new(),
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
}
