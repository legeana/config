use anyhow::{anyhow, Context, Result};

use crate::tag_criteria::TagCriteria;

use super::config;
use super::Installer;

#[derive(Default)]
pub struct UserDependency {
    installers: Vec<Box<dyn Installer>>,
}

impl UserDependency {
    pub fn new(cfg: &config::UserDependency) -> Result<Self> {
        let mut installers: Vec<Box<dyn Installer>> = Vec::new();
        if !cfg.is_satisfied().context("failed to check tags")? {
            return Ok(Self::default());
        }
        if let Some(_) = &cfg.brew {
            return Err(anyhow!("brew is not supported yet"));
        }
        if let Some(_) = &cfg.npm {
            return Err(anyhow!("npm is not supported yet"));
        }
        if let Some(_) = &cfg.npm {
            return Err(anyhow!("pip_user is not supported yet"));
        }
        if let Some(ansible_galaxy_role) = &cfg.ansible_galaxy_role {
            installers.push(Box::new(AnsibleGalaxy::new_role(
                ansible_galaxy_role.clone(),
            )));
        }
        if let Some(ansible_galaxy_collection) = &cfg.ansible_galaxy_role {
            installers.push(Box::new(AnsibleGalaxy::new_collection(
                ansible_galaxy_collection.clone(),
            )));
        }
        Ok(Self { installers })
    }
}

impl Installer for UserDependency {
    fn install(&self) -> Result<()> {
        self.installers.install()
    }
}

struct AnsibleGalaxy {
    galaxy_type: String,
    packages: Vec<String>,
}

impl AnsibleGalaxy {
    fn new_role(packages: Vec<String>) -> Self {
        Self {
            galaxy_type: "role".to_owned(),
            packages,
        }
    }
    fn new_collection(packages: Vec<String>) -> Self {
        Self {
            galaxy_type: "collection".to_owned(),
            packages,
        }
    }
}

impl Installer for AnsibleGalaxy {
    fn install(&self) -> Result<()> {
        let cmdline = format!(
            "ansible-galaxy {type} install -- {args}",
            type=self.galaxy_type,
            args=shlex::join(self.packages.iter().map(|s| s.as_ref()))
        );
        println!("$ {cmdline}");
        log::info!("Running $ {cmdline}");
        let status = std::process::Command::new("ansible-galaxy")
            .arg(&self.galaxy_type)
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
