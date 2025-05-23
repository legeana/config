use anyhow::{Context as _, Result};
use lontra_process::cmd;

use crate::command::is_command;
use crate::module::{Module, Rules};
use crate::tag_criteria::Criteria as _;

use super::Installer;
use super::config;
use super::satisficer::{DependencySatisficer, Satisficer as _};

#[derive(Default)]
pub(super) struct SystemDependency {
    wants: Option<DependencySatisficer>,
    installers: Vec<Box<dyn Installer>>,
}

impl SystemDependency {
    pub(super) fn new(cfg: &config::SystemDependency) -> Result<Self> {
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
        // FIXME: If a package is installed via user_dependencies, then this
        // will attempt to install package from a system dependency.
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
        cmd!(["sudo", "apt", "install", "--yes", "--"], &self.packages).run_verbose()
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
        cmd!(
            ["sudo", "pacman", "-S", "--needed", "--noconfirm", "--"],
            &self.packages
        )
        .run_verbose()
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
        cmd!(
            [
                "winget",
                "install",
                "--accept-package-agreements",
                "--accept-source-agreements",
                "--disable-interactivity",
                // Force reinstall. This is the only way to achieve success
                // code if the package is already installed.
                "--force",
                "--exact",
                "--source",
                &self.config.source,
                "--",
            ],
            self.config.packages.as_slice()
        )
        .run_verbose()
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
        cmd!(["bash", "-c", &self.script]).run_verbose()
    }
}
