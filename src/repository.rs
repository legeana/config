use std::collections::hash_map::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

pub struct Repository {
    root: PathBuf,
    name: String,
    packages: Vec<Package>,
}

pub struct SystemPackage {
    // TODO
}

pub trait Hook {
    // TODO
}

pub trait FileInstaller {
    // TODO
}

pub struct Configuration {
    root: PathBuf,
    subdirs: HashMap<String, Configuration>,
    pre_hooks: Vec<Box<dyn Hook>>,
    post_hooks: Vec<Box<dyn Hook>>,
    files: Vec<Box<dyn FileInstaller>>,
}

pub struct Package {
    name: String,
    configuration: Configuration,
    dependencies: Vec<String>,
    system_dependencies: Vec<SystemPackage>,
}

impl Repository {
    pub fn new(root: PathBuf) -> Result<Self> {
        let name: String = root
            .file_name()
            .ok_or(anyhow!("failed to get {} basename", root.display()))?
            .to_string_lossy()
            .into();
        let mut repository = Repository {
            root,
            name,
            packages: Vec::new(),
        };
        let dirs = repository
            .root
            .read_dir()
            .with_context(|| format!("failed to read {}", repository.root.display()))?;
        for entry in dirs {
            let dir = entry?;
            let package = Package::new(dir.path())
                .with_context(|| format!("failed to load {}", dir.path().display()))?;
            repository.packages.push(package);
        }
        return Ok(repository);
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn list(&self) -> Vec<String> {
        self.packages
            .iter()
            .map(|p| p.name().to_string())
            .collect()
    }
}

impl Package {
    fn new(root: PathBuf) -> Result<Self> {
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

impl Configuration {
    fn new(root: PathBuf) -> Result<Self> {
        Ok(Configuration {
            root,
            subdirs: HashMap::new(),
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
            files: Vec::new(),
        })
    }
}
