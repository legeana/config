use std::process::Command;

use anyhow::{anyhow, Context, Result};
use process_utils::cmd;

use crate::module::{Module, Rules};
use crate::tag_criteria::Criteria;

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
                config: cargo.to_cargo_config(),
            }));
        }
        if cfg.npm.is_some() {
            return Err(anyhow!("npm is not supported yet"));
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
        // FIXME: If a package is installed via system_dependencies, then this
        // will attempt to install package from a user dependency.
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
        cmd!(["brew", "tap", "--"], taps.as_slice()).run_verbose()
    }
    fn install_casks(&self) -> Result<()> {
        let Some(ref casks) = self.config.casks else {
            return Ok(());
        };
        cmd!(["brew", "install", "--cask", "--"], casks.as_slice()).run_verbose()
    }
    fn install_formulas(&self) -> Result<()> {
        let Some(ref formulas) = self.config.formulas else {
            return Ok(());
        };
        cmd!(["brew", "install", "--"], formulas.as_slice()).run_verbose()
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
    config: config::CargoConfig,
}

impl Installer for Cargo {
    fn install(&self, rules: &Rules) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("install");
        if rules.force_reinstall {
            cmd.arg("--force");
        }
        if let Some(git) = &self.config.git {
            cmd.arg("--git").arg(git);
        }
        if let Some(branch) = &self.config.branch {
            cmd.arg("--branch").arg(branch);
        }
        if let Some(tag) = &self.config.tag {
            cmd.arg("--tag").arg(tag);
        }
        if let Some(path) = &self.config.path {
            cmd.arg("--path").arg(path);
        }
        if self.config.locked.unwrap_or_default() {
            cmd.arg("--locked");
        }
        // Must be trailing arguments.
        cmd.arg("--");
        if let Some(crates) = &self.config.crates {
            cmd.args(crates.to_vec());
        }
        process_utils::run_verbose(&mut cmd)
    }
}

struct Pipx {
    packages: Vec<String>,
}

impl Installer for Pipx {
    fn install(&self, rules: &Rules) -> Result<()> {
        cmd!(
            ["pipx", "install",],
            rules.force_reinstall.then_some("--force"),
            ["--"],
            &self.packages
        )
        .run_verbose()
    }
}

struct Flatpak {
    config: config::FlatpakDependency,
}

impl Flatpak {
    fn clear_overrides(&self) -> Result<()> {
        cmd!([
            "flatpak",
            "--user",
            "override",
            "--reset",
            "--",
            &self.config.package
        ])
        .run_verbose()
    }
    fn set_overrides(&self) -> Result<()> {
        self.clear_overrides()?;
        let overrides = match &self.config.overrides {
            Some(o) => o.as_slice(),
            None => &[],
        };
        cmd!(
            ["flatpak", "--user", "override"],
            overrides,
            ["--", &self.config.package]
        )
        .run_verbose()
    }
    fn make_alias(&self) -> Result<()> {
        // TODO: need registry
        Ok(())
    }
}

impl Installer for Flatpak {
    fn install(&self, rules: &Rules) -> Result<()> {
        cmd!(
            ["flatpak", "--user", "install"],
            rules.force_update.then_some("--or-update"),
            rules.force_reinstall.then_some("--reinstall"),
            ["--", &self.config.repository, &self.config.package]
        )
        .run_verbose()?;
        self.set_overrides()?;
        self.make_alias()
    }
}
