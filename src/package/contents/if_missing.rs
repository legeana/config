use std::path::PathBuf;

use crate::module::{Module, Rules};

use super::parser;
use super::util;

use anyhow::{Context, Result};

pub struct IfMissingBuilder;

const COMMAND: &str = "if_missing";

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

impl parser::Builder for IfMissingBuilder {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "if_missing <path> <command> [<args>...]
           execute a MANIFEST <command> only if <path> is missing"
    }
    fn build(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (path, cmd_args) = util::multiple_args(COMMAND, args, 1)?;
        assert_eq!(path.len(), 1);
        let path: PathBuf = shellexpand::tilde(path[0]).as_ref().into();
        match parser::build(state, cmd_args)? {
            Some(cmd) => Ok(Some(Box::new(IfMissing { path, cmd }))),
            None => Ok(None),
        }
    }
}
