use anyhow::{anyhow, Context, Result};

use crate::tag_util;

use super::config;

trait Installer {
    fn install(&self) -> Result<()>;
}

#[derive(Default)]
pub struct SystemPackage {
    installers: Vec<Box<dyn Installer>>,
}

impl SystemPackage {
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
        // TODO apt
        // TODO brew
        // TODO npm
        if let Some(pacman) = &cfg.pacman {
            installers.push(Box::new(Pacman::new(pacman)));
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

struct Pacman {
    packages: Vec<String>,
}

impl Pacman {
    fn new(packages: &[String]) -> Self {
        Self {
            packages: packages.to_owned(),
        }
    }
}

impl Installer for Pacman {
    fn install(&self) -> Result<()> {
        let cmdline = format!(
            "sudo pacman -S --needed -- {}",
            shlex::join(self.packages.iter().map(|s| s.as_ref()))
        );
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
