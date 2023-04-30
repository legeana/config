use anyhow::{anyhow, Context, Result};

use crate::tag_util;

use super::config;
use super::Installer;

#[derive(Default)]
pub struct UserDependencyGroup {
    dependencies: Vec<UserDependency>,
}

impl UserDependencyGroup {
    pub fn new(cfg: &[config::UserDependency]) -> Result<Self> {
        let mut dependencies: Vec<UserDependency> = Vec::with_capacity(cfg.len());
        for dependency in cfg.iter() {
            dependencies.push(UserDependency::new(dependency)?);
        }
        Ok(Self { dependencies })
    }
    pub fn install(&self) -> Result<()> {
        for dependency in self.dependencies.iter() {
            dependency.install()?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct UserDependency {
    installers: Vec<Box<dyn Installer>>,
}

impl UserDependency {
    pub fn new(cfg: &config::UserDependency) -> Result<Self> {
        let installers: Vec<Box<dyn Installer>> = Vec::new();
        if let Some(requires) = &cfg.requires {
            if !tag_util::has_all_tags(requires)
                .with_context(|| format!("failed to check tags {requires:?}"))?
            {
                return Ok(Self::default());
            }
        }
        if let Some(conflicts) = &cfg.conflicts {
            if tag_util::has_any_tags(conflicts)
                .with_context(|| format!("failed to check tags {conflicts:?}"))?
            {
                return Ok(Self::default());
            }
        }
        if let Some(_) = &cfg.brew {
            return Err(anyhow!("brew is not supported yet"));
        }
        if let Some(_) = &cfg.npm {
            return Err(anyhow!("npm is not supported yet"));
        }
        if let Some(_) = &cfg.npm {
            return Err(anyhow!("pip_user is not supported yet"));
        }
        Ok(Self { installers })
    }
    pub fn install(&self) -> Result<()> {
        for installer in self.installers.iter() {
            installer.install()?
        }
        Ok(())
    }
}
