use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use crate::registry::Registry;

use super::Module;

pub struct AnsiblePlaybook {
    root: PathBuf,
    playbooks: Vec<String>,
    ask_become_pass: bool,
}

impl AnsiblePlaybook {
    pub fn new(root: PathBuf, playbooks: Vec<String>, ask_become_pass: bool) -> Self {
        Self {
            root,
            playbooks,
            ask_become_pass,
        }
    }
    fn run(&self) -> Result<()> {
        let flags = if self.ask_become_pass {
            vec!["--ask-become-pass".to_owned()]
        } else {
            Vec::<String>::default()
        };
        let cmdline = format!(
            "ansible-playbook {}{}-- {}",
            shlex::join(flags.iter().map(|s| s.as_ref())),
            if flags.is_empty() { "" } else { " " },
            shlex::join(self.playbooks.iter().map(|s| s.as_ref()))
        );
        println!("$ {cmdline}");
        log::info!("Running $ {cmdline}");
        let status = std::process::Command::new("ansible-playbook")
            .current_dir(&self.root)
            .args(&flags)
            .arg("--")
            .args(&self.playbooks)
            .status()
            .with_context(|| format!("failed to execute {cmdline:?}"))?;
        if !status.success() {
            return Err(anyhow!("failed to execute {cmdline:?}"));
        }
        Ok(())
    }
}

impl Module for AnsiblePlaybook {
    fn pre_install(&self, _: &mut dyn Registry) -> Result<()> {
        if self.ask_become_pass {
            return Ok(());
        }
        self.run()
    }
    fn system_install(&self) -> Result<()> {
        if !self.ask_become_pass {
            return Ok(());
        }
        self.run()
    }
}
