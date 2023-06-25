use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, Rules};

use super::builder;
use super::util;

fn is_command(exe: &str) -> Result<bool> {
    match which::which(exe) {
        Ok(_) => Ok(true),
        Err(which::Error::CannotFindBinaryPath) => Ok(false),
        Err(err) => Err(err).context(format!("failed to check if {exe:?} is available in PATH")),
    }
}

struct IfCommand {
    executable: String,
    cmd: Box<dyn Module>,
}

impl IfCommand {
    fn run<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        if !is_command(&self.executable)? {
            return Ok(());
        }
        f()
    }
}

impl Module for IfCommand {
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
struct IfCommandBuilder {
    executable: String,
    cmd: Box<dyn builder::Builder>,
}

impl builder::Builder for IfCommandBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        match self.cmd.build(state)? {
            Some(cmd) => Ok(Some(Box::new(IfCommand {
                executable: self.executable.clone(),
                cmd,
            }))),
            None => Ok(None),
        }
    }
}

#[derive(Clone)]
struct IfCommandParser;

impl builder::Parser for IfCommandParser {
    fn name(&self) -> String {
        "if_command".to_owned()
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
        Ok(Box::new(IfCommandBuilder {
            executable: exe[0].to_owned(),
            cmd: builder::parse(workdir, cmd_args)?,
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(IfCommandParser {})]
}
