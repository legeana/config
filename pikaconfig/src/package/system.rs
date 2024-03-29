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
            installers.push(Box::new(Apt::new(apt)));
        }
        if let Some(pacman) = cfg.pacman.clone().or_else(|| cfg.any.clone()) {
            installers.push(Box::new(Pacman::new(pacman)));
        }
        if let Some(winget) = cfg.winget.clone().or_else(|| cfg.any.clone()) {
            installers.push(Box::new(Winget::new(winget)));
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
        if !rules.force_download && self.wants.is_satisfied()? {
            return Ok(());
        }
        self.installers.install()
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
    fn install(&self) -> Result<()> {
        if !is_command("apt")? {
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
    fn install(&self) -> Result<()> {
        if !is_command("pacman")? {
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
    packages: Vec<String>,
}

impl Winget {
    fn new(packages: Vec<String>) -> Self {
        Self { packages }
    }
}

impl Installer for Winget {
    fn install(&self) -> Result<()> {
        if !is_command("winget")? {
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
                .arg("--")
                .args(&self.packages),
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
    fn install(&self) -> Result<()> {
        process_utils::run_verbose(
            std::process::Command::new("bash")
                .arg("-c")
                .arg(&self.script),
        )
    }
}
