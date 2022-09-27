use std::ffi::OsStr;

use anyhow::{anyhow, Context, Result};

use crate::tag_util;

use super::config;

trait Installer {
    fn install(&self) -> Result<()>;
}

#[derive(Default)]
pub struct SystemDependency {
    variants: Vec<SystemDependencyVariant>,
}

impl SystemDependency {
    pub fn new(cfg: &[config::SystemDependency]) -> Result<Self> {
        let mut variants: Vec<SystemDependencyVariant> = Vec::with_capacity(cfg.len());
        for variant in cfg.iter() {
            variants.push(SystemDependencyVariant::new(variant)?);
        }
        Ok(SystemDependency { variants })
    }
    pub fn install(&self) -> Result<()> {
        for variant in self.variants.iter() {
            variant.install()?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct SystemDependencyVariant {
    installers: Vec<Box<dyn Installer>>,
}

impl SystemDependencyVariant {
    pub fn new(cfg: &config::SystemDependency) -> Result<Self> {
        let if_available = cfg.if_available.unwrap_or_default();
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
        if let Some(apt) = &cfg.apt {
            installers.push(Box::new(Apt::new(apt.clone(), if_available)));
        }
        // TODO brew
        // TODO npm
        if let Some(pacman) = &cfg.pacman {
            installers.push(Box::new(Pacman::new(pacman.clone(), if_available)));
        }
        // TODO pip_user
        if let Some(exec) = &cfg.exec {
            installers.push(Box::new(Exec::new(exec).with_context(|| {
                format!("failed to parse system_dependencies.exec {exec:?}")
            })?));
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
    if_available: bool,
}

impl Apt {
    fn new(packages: Vec<String>, if_available: bool) -> Self {
        Self {
            packages,
            if_available,
        }
    }
}

impl Installer for Apt {
    fn install(&self) -> Result<()> {
        if self.if_available && !is_available("apt")? {
            return Ok(());
        }
        let cmdline = format!(
            "sudo apt install -- {}",
            shlex::join(self.packages.iter().map(|s| s.as_ref()))
        );
        println!("$ {cmdline}");
        log::info!("Running $ {cmdline}");
        let status = std::process::Command::new("sudo")
            .arg("apt")
            .arg("install")
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
    if_available: bool,
}

impl Pacman {
    fn new(packages: Vec<String>, if_available: bool) -> Self {
        Self {
            packages,
            if_available,
        }
    }
}

impl Installer for Pacman {
    fn install(&self) -> Result<()> {
        if self.if_available && !is_available("pacman")? {
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

struct Exec {
    #[allow(dead_code)]
    commands: Vec<Vec<String>>,
}

impl Exec {
    fn new(_cmd: &str) -> Result<Self> {
        unimplemented!()
    }
}

impl Installer for Exec {
    fn install(&self) -> Result<()> {
        unimplemented!()
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
