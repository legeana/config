use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::module::{Module, Rules};
use crate::tag_criteria::Criteria;

use crate::process_utils;

use super::config;
use super::satisficer::{self, Satisficer};
use super::Installer;

#[derive(Default)]
pub struct UserDependency {
    satisficer: Option<Box<dyn Satisficer>>,
    installers: Vec<Box<dyn Installer>>,
}

impl UserDependency {
    pub fn new(cfg: &config::UserDependency) -> Result<Self> {
        if !cfg.is_satisfied().context("failed to check tags")? {
            return Ok(Self::default());
        }
        let satisficer: Option<Box<dyn Satisficer>> = match &cfg.wants {
            Some(config::Satisficer::Command { command }) => {
                Some(Box::new(satisficer::WantsCommand::new(command)))
            }
            None => None,
        };
        let mut installers: Vec<Box<dyn Installer>> = Vec::new();
        if cfg.brew.is_some() {
            return Err(anyhow!("brew is not supported yet"));
        }
        if let Some(cargo) = &cfg.cargo {
            installers.push(Box::new(Cargo {
                packages: cargo.clone(),
            }));
        }
        if cfg.npm.is_some() {
            return Err(anyhow!("npm is not supported yet"));
        }
        if cfg.npm.is_some() {
            return Err(anyhow!("pip_user is not supported yet"));
        }
        Ok(Self {
            satisficer,
            installers,
        })
    }
}

impl Module for UserDependency {
    fn pre_uninstall(&self, rules: &Rules) -> Result<()> {
        if !rules.force_download && self.satisficer.is_satisfied()? {
            return Ok(());
        }
        self.installers.install()
    }
}

struct Cargo {
    packages: Vec<String>,
}

impl Installer for Cargo {
    fn install(&self) -> Result<()> {
        process_utils::run_verbose(
            Command::new("cargo")
                .arg("install")
                .arg("--")
                .args(&self.packages),
        )
    }
}
