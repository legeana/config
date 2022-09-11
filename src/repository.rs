use std::path::PathBuf;

use crate::package::Package;
use crate::registry::Registry;

use anyhow::{anyhow, Context, Result};

pub struct Repository {
    root: PathBuf,
    name: String,
    packages: Vec<Package>,
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
            let md = std::fs::metadata(dir.path())
                .with_context(|| format!("failed to read metadata for {}", dir.path().display()))?;
            if !md.is_dir() {
                continue;
            }
            let package = Package::new(dir.path())
                .with_context(|| format!("failed to load {}", dir.path().display()))?;
            repository.packages.push(package);
        }
        repository.packages.sort_by(|a, b| a.name().cmp(b.name()));
        return Ok(repository);
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn list(&self) -> Vec<String> {
        self.packages.iter().map(|p| p.name().to_string()).collect()
    }
    pub fn pre_install_all(&self) -> Result<()> {
        for package in self.packages.iter() {
            package
                .pre_install()
                .with_context(|| format!("failed to pre-install {}", package.name()))?;
        }
        return Ok(());
    }
    pub fn install_all(&self, registry: &mut dyn Registry) -> Result<()> {
        for package in self.packages.iter() {
            package
                .install(registry)
                .with_context(|| format!("failed to install {}", package.name()))?;
        }
        return Ok(());
    }
    pub fn post_install_all(&self) -> Result<()> {
        for package in self.packages.iter() {
            package
                .post_install()
                .with_context(|| format!("failed to post-install {}", package.name()))?;
        }
        return Ok(());
    }
}
