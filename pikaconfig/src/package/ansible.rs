use std::path::PathBuf;

use anyhow::Result;

use crate::module::{Module, Rules};
use crate::process_utils;
use crate::registry::Registry;

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
        process_utils::run_verbose(
            std::process::Command::new("ansible-playbook")
                .current_dir(&self.root)
                .args(&flags)
                .arg("--")
                .args(&self.playbooks),
        )
    }
}

impl Module for AnsiblePlaybook {
    fn post_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        // Use post_install because MANIFEST should not have much logic if
        // ansible is involved anyway. Install/link files first, then let
        // ansible handle the rest.
        if self.ask_become_pass {
            return Ok(());
        }
        self.run()
    }
    fn system_install(&self, _rules: &Rules) -> Result<()> {
        if !self.ask_become_pass {
            return Ok(());
        }
        self.run()
    }
}
