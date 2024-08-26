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
        if let Some(brew) = &cfg.brew {
            installers.push(Box::new(Brew {
                config: brew.to_config(),
            }));
        }
        if let Some(cargo) = &cfg.cargo {
            installers.push(Box::new(Cargo {
                config: cargo.clone(),
            }));
        }
        if cfg.npm.is_some() {
            return Err(anyhow!("npm is not supported yet"));
        }
        if cfg.pip_user.is_some() {
            return Err(anyhow!("pip_user is not supported yet"));
        }
        if let Some(pipx) = &cfg.pipx {
            installers.push(Box::new(Pipx {
                packages: pipx.to_vec(),
            }));
        }
        if let Some(flatpak) = &cfg.flatpak {
            installers.push(Box::new(Flatpak {
                config: flatpak.clone(),
            }));
        }
        if cfg.binary_url.is_some() {
            return Err(anyhow!("binary_url is not supported yet"));
        }
        if cfg.github_release.is_some() {
            return Err(anyhow!("github_release is not supported yet"));
        }
        Ok(Self {
            wants: cfg.wants.clone(),
            installers,
        })
    }
}

impl Module for UserDependency {
    fn pre_uninstall(&self, rules: &Rules) -> Result<()> {
        if !rules.force_update && self.wants.is_satisfied()? {
            return Ok(());
        }
        self.installers.install(rules)
    }
}

struct Brew {
    config: config::BrewConfig,
}

impl Brew {
    fn install_taps(&self) -> Result<()> {
        let Some(ref taps) = self.config.taps else {
            return Ok(());
        };
        let mut cmd = Command::new("brew");
        cmd.arg("tap");
        cmd.arg("--");
        cmd.args(taps.as_slice());
        process_utils::run_verbose(&mut cmd)
    }
    fn install_casks(&self) -> Result<()> {
        let Some(ref casks) = self.config.casks else {
            return Ok(());
        };
        let mut cmd = Command::new("brew");
        cmd.arg("install");
        cmd.arg("--cask");
        cmd.arg("--");
        cmd.args(casks.as_slice());
        process_utils::run_verbose(&mut cmd)
    }
    fn install_formulas(&self) -> Result<()> {
        let Some(ref formulas) = self.config.formulas else {
            return Ok(());
        };
        let mut cmd = Command::new("brew");
        cmd.arg("install");
        cmd.arg("--");
        cmd.args(formulas.as_slice());
        process_utils::run_verbose(&mut cmd)
    }
}

impl Installer for Brew {
    fn install(&self, _rules: &Rules) -> Result<()> {
        self.install_taps()?;
        self.install_casks()?;
        self.install_formulas()?;
        Ok(())
    }
}

struct Cargo {
    config: config::CargoDependency,
}

impl Installer for Cargo {
    fn install(&self, rules: &Rules) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("install");
        if rules.force_reinstall {
            cmd.arg("--force");
        }
        match &self.config {
            config::CargoDependency::Crates(packages) => {
                cmd.arg("--").args(packages.to_vec());
            }
            config::CargoDependency::Config {
                crates,
                git,
                branch,
                tag,
                path,
                locked,
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
                if locked.unwrap_or_default() {
                    cmd.arg("--locked");
                }
                // Must be trailing arguments.
                cmd.arg("--");
                if let Some(crates) = crates {
                    cmd.args(crates.to_vec());
                }
            }
        }
        process_utils::run_verbose(&mut cmd)
    }
}

struct Pipx {
    packages: Vec<String>,
}

impl Installer for Pipx {
    fn install(&self, rules: &Rules) -> Result<()> {
        let mut cmd = Command::new("pipx");
        cmd.arg("install");
        if rules.force_reinstall {
            cmd.arg("--force");
        }
        cmd.arg("--");
        cmd.args(&self.packages);
        process_utils::run_verbose(&mut cmd)
    }
}

struct Flatpak {
    config: config::FlatpakDependency,
}

impl Flatpak {
    fn clear_overrides(&self) -> Result<()> {
        let mut cmd = Command::new("flatpak");
        cmd.arg("--user");
        cmd.arg("override");
        cmd.arg("--reset");
        cmd.arg("--");
        cmd.arg(&self.config.package);
        process_utils::run_verbose(&mut cmd)
    }
    fn set_overrides(&self) -> Result<()> {
        self.clear_overrides()?;
        let mut cmd = Command::new("flatpak");
        cmd.arg("--user");
        cmd.arg("override");
        if let Some(overrides) = &self.config.overrides {
            cmd.args(overrides.as_slice());
        }
        cmd.arg("--");
        cmd.arg(&self.config.package);
        process_utils::run_verbose(&mut cmd)
    }
    fn make_alias(&self) -> Result<()> {
        // TODO: need registry
        Ok(())
    }
}

impl Installer for Flatpak {
    fn install(&self, rules: &Rules) -> Result<()> {
        let mut cmd = Command::new("flatpak");
        cmd.arg("--user");
        cmd.arg("install");
        if rules.force_update {
            cmd.arg("--or-update");
        }
        if rules.force_reinstall {
            cmd.arg("--reinstall");
        }
        cmd.arg("--");
        cmd.arg(&self.config.repository);
        cmd.arg(&self.config.package);
        process_utils::run_verbose(&mut cmd)?;
        self.set_overrides()?;
        self.make_alias()
    }
}
