use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::module::{Module, Rules};
use crate::tag_criteria::Criteria;

use crate::process_utils;

use super::config;
use super::satisficer::{DependencySatisficer, Satisficer};
use super::Installer;

#[derive(Default)]
pub struct UserDependency {
    wants: Option<DependencySatisficer>,
    installers: Vec<Box<dyn Installer>>,
}

impl UserDependency {
    pub fn new(cfg: &config::UserDependency) -> Result<Self> {
        if !cfg
            .requires
            .is_satisfied()
            .context("failed to check tags")?
        {
            return Ok(Self::default());
        }
        let mut installers: Vec<Box<dyn Installer>> = Vec::new();
        if cfg.brew.is_some() {
            return Err(anyhow!("brew is not supported yet"));
        }
        if let Some(cargo) = &cfg.cargo {
            installers.push(Box::new(Cargo {
                config: cargo.clone(),
            }));
        }
        if cfg.npm.is_some() {
            return Err(anyhow!("npm is not supported yet"));
        }
        if cfg.npm.is_some() {
            return Err(anyhow!("pip_user is not supported yet"));
        }
        Ok(Self {
            wants: cfg.wants.clone(),
            installers,
        })
    }
}

impl Module for UserDependency {
    fn pre_uninstall(&self, rules: &Rules) -> Result<()> {
        if !rules.force_download && self.wants.is_satisfied()? {
            return Ok(());
        }
        self.installers.install()
    }
}

struct Cargo {
    config: config::CargoDependency,
}

impl Installer for Cargo {
    fn install(&self) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("install");
        match &self.config {
            config::CargoDependency::Crates(packages) => {
                cmd.arg("--").args(packages);
            }
            config::CargoDependency::Config {
                crates,
                git,
                branch,
                tag,
                path,
            } => {
                if let Some(git) = git {
                    cmd.arg("--git").arg(git);
                }
                if let Some(branch) = branch {
                    cmd.arg("--branch").arg(branch);
                }
                if let Some(tag) = tag {
                    cmd.arg("--tag").arg(tag);
                }
                if let Some(path) = path {
                    cmd.arg("--path").arg(path);
                }
                // Must be trailing arguments.
                cmd.arg("--");
                if let Some(crates) = crates {
                    cmd.args(crates);
                }
            }
        }
        process_utils::run_verbose(&mut cmd)
    }
}
