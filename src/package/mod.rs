mod configuration;
mod system;

use std::path::PathBuf;

use configuration::Configuration;
use system::SystemPackage;

use anyhow::{anyhow, Result};

pub use configuration::help as manifest_help;

pub struct Package {
    name: String,
    configuration: Configuration,
    dependencies: Vec<String>,
    system_dependencies: Vec<SystemPackage>,
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
}
