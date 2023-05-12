mod config;

use std::ffi::OsStr;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use crate::package::{Module, Package};
use crate::registry::Registry;
use crate::tag_criteria::TagCriteria;

pub use config::is_repository_dir;

pub struct Repository {
    root: PathBuf,
    name: String,
    config: config::Repository,
    packages: Vec<Package>,
}

impl Repository {
    pub fn new(root: PathBuf) -> Result<Self> {
        let name: String = root
            .file_name()
            .ok_or_else(|| anyhow!("failed to get {root:?} basename"))?
            .to_string_lossy()
            .into();
        let config = config::load_repository(&root)
            .with_context(|| format!("failed to load repository {root:?} config"))?;
        let mut repository = Repository {
            root,
            name,
            config,
            packages: Vec::new(),
        };
        let dirs = repository
            .root
            .read_dir()
            .with_context(|| format!("failed to read {:?}", repository.root))?;
        for entry in dirs {
            let dir = entry?;
            if dir.path().file_name() == Some(OsStr::new(".git")) {
                continue;
            }
            let md = std::fs::metadata(dir.path())
                .with_context(|| format!("failed to read metadata for {:?}", dir.path()))?;
            if !md.is_dir() {
                continue;
            }
            let package = Package::new(dir.path())
                .with_context(|| format!("failed to load {:?}", dir.path()))?;
            repository.packages.push(package);
        }
        repository.packages.sort_by(|a, b| a.name().cmp(b.name()));
        Ok(repository)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn list(&self) -> Vec<String> {
        self.packages.iter().map(|p| p.name().to_string()).collect()
    }
    pub fn enabled(&self) -> Result<bool> {
        if !self.config.is_satisfied().context("failed to check tags")? {
            return Ok(false);
        }
        Ok(true)
    }
    pub fn pre_install_all(&self, registry: &mut dyn Registry) -> Result<()> {
        if !self.enabled()? {
            return Ok(());
        }
        for package in self.packages.iter() {
            package
                .pre_install(registry)
                .with_context(|| format!("failed to pre-install {}", package.name()))?;
        }
        Ok(())
    }
    pub fn install_all(&self, registry: &mut dyn Registry) -> Result<()> {
        if !self.enabled()? {
            return Ok(());
        }
        for package in self.packages.iter() {
            package
                .install(registry)
                .with_context(|| format!("failed to install {}", package.name()))?;
        }
        Ok(())
    }
    pub fn post_install_all(&self, registry: &mut dyn Registry) -> Result<()> {
        if !self.enabled()? {
            return Ok(());
        }
        for package in self.packages.iter() {
            package
                .post_install(registry)
                .with_context(|| format!("failed to post-install {}", package.name()))?;
        }
        Ok(())
    }
    pub fn system_install_all(&self, strict: bool) -> Result<()> {
        if !self.enabled()? {
            return Ok(());
        }
        for package in self.packages.iter() {
            let result = package
                .system_install()
                .with_context(|| format!("failed to system install {}", package.name()));
            match result {
                Ok(_) => (),
                Err(err) => {
                    if strict {
                        return Err(err);
                    } else {
                        log::error!("failed to install {}: {err}", package.name());
                    }
                }
            };
        }
        Ok(())
    }
}
