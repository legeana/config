use std::path::PathBuf;

use crate::module::{Module, Rules};

use super::builder;
use super::util;

use anyhow::{Context, Result};
use indoc::formatdoc;

pub struct IfMissingBuilder;

struct IfMissing {
    path: PathBuf,
    cmd: Box<dyn Module>,
}

impl IfMissing {
    fn run<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        if self
            .path
            .try_exists()
            .with_context(|| format!("failed to check if {:?} exists", &self.path))?
        {
            return Ok(());
        }
        f()
    }
}

impl Module for IfMissing {
    fn pre_install(
        &self,
        rules: &Rules,
        registry: &mut dyn crate::registry::Registry,
    ) -> Result<()> {
        self.run(|| self.cmd.pre_install(rules, registry))
    }
    fn install(&self, rules: &Rules, registry: &mut dyn crate::registry::Registry) -> Result<()> {
        self.run(|| self.cmd.install(rules, registry))
    }
    fn post_install(
        &self,
        rules: &Rules,
        registry: &mut dyn crate::registry::Registry,
    ) -> Result<()> {
        self.run(|| self.cmd.post_install(rules, registry))
    }
}

impl builder::Builder for IfMissingBuilder {
    fn name(&self) -> String {
        "if_missing".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <path> <command> [<args>...]
                execute a MANIFEST <command> only if <path> is missing
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (path, cmd_args) = util::multiple_args(&self.name(), args, 1)?;
        assert_eq!(path.len(), 1);
        let path: PathBuf = shellexpand::tilde(path[0]).as_ref().into();
        match builder::build(state, cmd_args)? {
            Some(cmd) => Ok(Some(Box::new(IfMissing { path, cmd }))),
            None => Ok(None),
        }
    }
}
