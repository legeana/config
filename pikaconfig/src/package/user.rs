use anyhow::{anyhow, Context, Result};

use crate::module::{Module, Rules};
use crate::registry::Registry;
use crate::tag_criteria::TagCriteria;

use super::config;
use super::Installer;

#[derive(Default)]
pub struct UserDependency {
    installers: Vec<Box<dyn Installer>>,
}

impl UserDependency {
    pub fn new(cfg: &config::UserDependency) -> Result<Self> {
        let installers: Vec<Box<dyn Installer>> = Vec::new();
        if !cfg.is_satisfied().context("failed to check tags")? {
            return Ok(Self::default());
        }
        if cfg.brew.is_some() {
            return Err(anyhow!("brew is not supported yet"));
        }
        if cfg.npm.is_some() {
            return Err(anyhow!("npm is not supported yet"));
        }
        if cfg.npm.is_some() {
            return Err(anyhow!("pip_user is not supported yet"));
        }
        Ok(Self { installers })
    }
}

impl Module for UserDependency {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        self.installers.install()
    }
}
