use std::ffi::OsStr;

use anyhow::{anyhow, Context, Result};

use crate::tag_util;

use super::config;
use super::Installer;

#[derive(Default)]
pub struct SystemDependencyGroup {
    dependencies: Vec<SystemDependency>,
}

impl SystemDependencyGroup {
    pub fn new(cfg: &[config::SystemDependency]) -> Result<Self> {
        let mut dependencies: Vec<SystemDependency> = Vec::with_capacity(cfg.len());
        for dependency in cfg.iter() {
            dependencies.push(SystemDependency::new(dependency)?);
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
pub struct SystemDependency {
    installers: Vec<Box<dyn Installer>>,
}

impl SystemDependency {
    pub fn new(cfg: &config::SystemDependency) -> Result<Self> {
        let mut installers: Vec<Box<dyn Installer>> = Vec::new();
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
        if let Some(apt) = cfg.apt.clone().or_else(|| cfg.any.clone()) {
            installers.push(Box::new(Apt::new(apt)));
        }
        if let Some(pacman) = cfg.pacman.clone().or_else(|| cfg.any.clone()) {
            installers.push(Box::new(Pacman::new(pacman)));
        }
        if let Some(bash) = &cfg.bash {
            installers.push(Box::new(Bash::new(bash.clone())));
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
        if !is_available("apt")? {
            return Ok(());
        }
        let cmdline = format!(
            "sudo apt install --yes -- {}",
            shlex::join(self.packages.iter().map(|s| s.as_ref()))
        );
        println!("$ {cmdline}");
        log::info!("Running $ {cmdline}");
        let status = std::process::Command::new("sudo")
            .arg("apt")
            .arg("install")
            .arg("--yes")
            .arg("--")
            .args(&self.packages)
            .status()
            .with_context(|| format!("failed to execute {cmdline:?}"))?;
        if !status.success() {
            return Err(anyhow!("failed to execute {cmdline:?}"));
        }
        Ok(())
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
        if !is_available("pacman")? {
            return Ok(());
        }
        let cmdline = format!(
            "sudo pacman -S --needed -- {}",
            shlex::join(self.packages.iter().map(|s| s.as_ref()))
        );
        println!("$ {cmdline}");
        log::info!("Running $ {cmdline}");
        let status = std::process::Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .arg("--needed")
            .arg("--")
            .args(&self.packages)
            .status()
            .with_context(|| format!("failed to execute {cmdline:?}"))?;
        if !status.success() {
            return Err(anyhow!("failed to execute {cmdline:?}"));
        }
        Ok(())
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
        let cmdline = format!("bash -c {}", shlex::quote(&self.script));
        println!("$ {cmdline}");
        log::info!("Running $ {cmdline}");
        let status = std::process::Command::new("bash")
            .arg("-c")
            .arg(&self.script)
            .status()
            .context("failed to execute bash")?;
        if !status.success() {
            return Err(anyhow!("failed to execute {cmdline:?}"));
        }
        Ok(())
    }
}

fn is_available<T: AsRef<OsStr> + std::fmt::Debug>(cmd: T) -> Result<bool> {
    match which::which(&cmd) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err {
                which::Error::CannotFindBinaryPath => Ok(false),
                _ => Err(anyhow::Error::new(err)
                    .context(format!("failed to check if {cmd:?} is installed"))),
            }
        }
    }
}
