use std::path::PathBuf;

use crate::module::{Module, Rules};

use super::builder;
use super::util;

use anyhow::{Context, Result};
use indoc::formatdoc;

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

#[derive(Debug)]
struct IfMissingBuilder {
    path: String,
    // TODO cmd: Box<dyn Module>
    cmd_args: Vec<String>,
}

impl builder::Builder for IfMissingBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let path: PathBuf = shellexpand::tilde(&self.path).as_ref().into();
        let cmd_args: Vec<_> = self.cmd_args.iter().map(String::as_str).collect();
        match builder::build(state, &cmd_args)? {
            Some(cmd) => Ok(Some(Box::new(IfMissing { path, cmd }))),
            None => Ok(None),
        }
    }
}

#[derive(Clone)]
struct IfMissingParser;

impl builder::Parser for IfMissingParser {
    fn name(&self) -> String {
        "if_missing".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <path> <command> [<args>...]
                execute a MANIFEST <command> only if <path> is missing
        ", command=self.name()}
    }
    fn parse(&self, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let (path, cmd_args) = util::multiple_args(&self.name(), args, 1)?;
        assert_eq!(path.len(), 1);
        Ok(Box::new(IfMissingBuilder {
            path: path[0].to_owned(),
            cmd_args: cmd_args.iter().map(|&s| s.to_owned()).collect(),
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(IfMissingParser {})]
}
