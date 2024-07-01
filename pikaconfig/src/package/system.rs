use anyhow::{Context, Result};

use crate::command::is_command;
use crate::module::{Module, Rules};
use crate::process_utils;
use crate::tag_criteria::Criteria;

use super::config;
use super::satisficer::{DependencySatisficer, Satisficer};
use super::Installer;

#[derive(Default)]
pub struct SystemDependency {
    wants: Option<DependencySatisficer>,
    installers: Vec<Box<dyn Installer>>,
}

impl SystemDependency {
    pub fn new(cfg: &config::SystemDependency) -> Result<Self> {
        let mut installers: Vec<Box<dyn Installer>> = Vec::new();
        if !cfg
            .requires
            .is_satisfied()
            .context("failed to check tags")?
        {
            return Ok(Self::default());
        }
        if let Some(apt) = cfg.apt.clone().or_else(|| cfg.any.clone()) {
            installers.push(Box::new(Apt::new(apt.to_vec())));
        }
        if let Some(pacman) = cfg.pacman.clone().or_else(|| cfg.any.clone()) {
            installers.push(Box::new(Pacman::new(pacman.to_vec())));
        }
        //if let Some(winget) = cfg.winget.clone().map_or_else(|dep| dep.to_config(), || )
        if let Some(winget) = cfg
            .winget
            .clone()
            .or_else(|| cfg.any.clone().map(config::WingetDependency::WingetSource))
        {
            installers.push(Box::new(Winget {
                config: winget.to_config(),
            }));
        }
        if let Some(bash) = &cfg.bash {
            installers.push(Box::new(Bash::new(bash.clone())));
        }
        Ok(Self {
            wants: cfg.wants.clone(),
            installers,
        })
    }
}

impl Module for SystemDependency {
    fn system_install(&self, rules: &Rules) -> Result<()> {
        if !rules.force_update && self.wants.is_satisfied()? {
            return Ok(());
        }
        self.installers.install(rules)
    }
}

struct Apt {
    packages: Vec<String>,
}

impl Apt {
    fn new(packages: Vec<String>) -> Self {
        Self { packages }
    }
}

impl Installer for Apt {
    fn install(&self, _rules: &Rules) -> Result<()> {
        if !is_command("apt")? {
            return Ok(());
        }
        if self.packages.is_empty() {
            return Ok(());
        }
        process_utils::run_verbose(
            std::process::Command::new("sudo")
                .arg("apt")
                .arg("install")
                .arg("--yes")
                .arg("--")
                .args(&self.packages),
        )
    }
}

struct Pacman {
    packages: Vec<String>,
}

impl Pacman {
    fn new(packages: Vec<String>) -> Self {
        Self { packages }
    }
}

impl Installer for Pacman {
    fn install(&self, _rules: &Rules) -> Result<()> {
        if !is_command("pacman")? {
            return Ok(());
        }
        if self.packages.is_empty() {
            return Ok(());
        }
        process_utils::run_verbose(
            std::process::Command::new("sudo")
                .arg("pacman")
                .arg("-S")
                .arg("--needed")
                .arg("--noconfirm")
                .arg("--")
                .args(&self.packages),
        )
    }
}

struct Winget {
    config: config::WingetConfig,
}

impl Installer for Winget {
    fn install(&self, _rules: &Rules) -> Result<()> {
        if !is_command("winget")? {
            return Ok(());
        }
        if self.config.packages.is_empty() {
            return Ok(());
        }
        process_utils::run_verbose(
            std::process::Command::new("winget")
                .arg("install")
                .arg("--accept-package-agreements")
                .arg("--accept-source-agreements")
                .arg("--disable-interactivity")
                // Force reinstall. This is the only way to achieve success
                // code if the package is already installed.
                .arg("--force")
                .arg("--exact")
                .arg("--source")
                .arg(&self.config.source)
                .arg("--")
                .args(self.config.packages.as_slice()),
        )
    }
}

struct Bash {
    script: String,
}

impl Bash {
    fn new(script: String) -> Self {
        Self { script }
    }
}

impl Installer for Bash {
    fn install(&self, _rules: &Rules) -> Result<()> {
        process_utils::run_verbose(
            std::process::Command::new("bash")
                .arg("-c")
                .arg(&self.script),
        )
    }
}
