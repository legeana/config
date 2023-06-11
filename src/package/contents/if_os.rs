use crate::module::{Module, Rules};

use super::builder;
use super::util;

use anyhow::Result;
use indoc::formatdoc;

struct IfOs {
    want_os: &'static str,
    cmd: Box<dyn Module>,
}

impl IfOs {
    fn is_os(&self) -> bool {
        self.want_os == std::env::consts::FAMILY || self.want_os == std::env::consts::OS
    }
    fn run<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        if !self.is_os() {
            return Ok(());
        }
        f()
    }
}

impl Module for IfOs {
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

struct IfOsBuilder {
    os: &'static str,
}

impl IfOsBuilder {
    fn command(&self) -> String {
        format!("if_{}", self.os)
    }
}

impl builder::Builder for IfOsBuilder {
    fn name(&self) -> String {
        self.command()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <command> [<args>...]
                execute a MANIFEST <command> only if os (or family) is {os}
        ", os=self.os, command=self.command()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (empty, cmd_args) = util::multiple_args(&self.command(), args, 0)?;
        assert!(empty.is_empty());
        match builder::build(state, cmd_args)? {
            Some(cmd) => Ok(Some(Box::new(IfOs {
                want_os: self.os,
                cmd,
            }))),
            None => Ok(None),
        }
    }
}

pub fn if_macos() -> Box<dyn builder::Builder> {
    Box::new(IfOsBuilder { os: "macos" })
}

pub fn if_linux() -> Box<dyn builder::Builder> {
    Box::new(IfOsBuilder { os: "linux" })
}

pub fn if_unix() -> Box<dyn builder::Builder> {
    Box::new(IfOsBuilder { os: "unix" })
}

pub fn if_windows() -> Box<dyn builder::Builder> {
    Box::new(IfOsBuilder { os: "windows" })
}
