use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, Rules};

use super::builder;
use super::util;

struct IfExecutable {
    executable: String,
    cmd: Box<dyn Module>,
}

impl IfExecutable {
    fn is_available(&self) -> Result<bool> {
        match which::which(&self.executable) {
            Ok(_) => Ok(true),
            Err(which::Error::CannotFindBinaryPath) => Ok(false),
            Err(err) => Err(err).context(format!(
                "failed to check if {:?} is available in PATH",
                self.executable
            )),
        }
    }
    fn run<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        if !self.is_available()? {
            return Ok(());
        }
        f()
    }
}

impl Module for IfExecutable {
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
struct IfExecutableBuilder {
    executable: String,
    cmd: Box<dyn builder::Builder>,
}

impl builder::Builder for IfExecutableBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        match self.cmd.build(state)? {
            Some(cmd) => Ok(Some(Box::new(IfExecutable {
                executable: self.executable.clone(),
                cmd,
            }))),
            None => Ok(None),
        }
    }
}

#[derive(Clone)]
struct IfExecutableParser;

impl builder::Parser for IfExecutableParser {
    fn name(&self) -> String {
        "if_executable".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <executable> <command> [<args>...]
                execute a MANIFEST <command> only if <executable> is available
                in PATH
        ", command=self.name()}
    }
    fn parse(&self, workdir: &std::path::Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let (exe, cmd_args) = util::multiple_args(&self.name(), args, 1)?;
        assert_eq!(exe.len(), 1);
        Ok(Box::new(IfExecutableBuilder {
            executable: exe[0].to_owned(),
            cmd: builder::parse(workdir, cmd_args)?,
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(IfExecutableParser {})]
}
