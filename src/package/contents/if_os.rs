use crate::module::Module;

use super::builder;
use super::util;

use anyhow::Result;
use indoc::formatdoc;

#[derive(Debug)]
struct IfOsBuilder {
    os: &'static str,
    cmd: Box<dyn builder::Builder>,
}

impl IfOsBuilder {
    fn is_os(&self) -> bool {
        // We don't have to check this in runtime as this never changes.
        // In fact, it's even beneficial to check this during build to support *Prefix.
        self.os == std::env::consts::FAMILY || self.os == std::env::consts::OS
    }
}

impl builder::Builder for IfOsBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        if !self.is_os() {
            return Ok(None);
        }
        self.cmd.build(state)
    }
}

#[derive(Clone)]
struct IfOsParser {
    os: &'static str,
}

impl IfOsParser {
    fn command(&self) -> String {
        format!("if_{}", self.os)
    }
}

impl builder::Parser for IfOsParser {
    fn name(&self) -> String {
        self.command()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <command> [<args>...]
                execute a MANIFEST <command> only if os (or family) is {os}
        ", os=self.os, command=self.command()}
    }
    fn parse(&self, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let (empty, cmd_args) = util::multiple_args(&self.command(), args, 0)?;
        assert!(empty.is_empty());
        Ok(Box::new(IfOsBuilder {
            os: self.os,
            cmd: builder::parse(cmd_args)?,
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![
        Box::new(IfOsParser { os: "macos" }),
        Box::new(IfOsParser { os: "linux" }),
        Box::new(IfOsParser { os: "unix" }),
        Box::new(IfOsParser { os: "windows" }),
    ]
}
