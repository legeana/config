use crate::module::Module;

use super::builder;
use super::util;

use anyhow::Result;
use indoc::formatdoc;

struct IfOsBuilder {
    os: &'static str,
}

impl IfOsBuilder {
    fn is_os(&self) -> bool {
        // We don't have to check this in runtime as this never changes.
        // In fact, it's even beneficial to check this during build to support *Prefix.
        self.os == std::env::consts::FAMILY || self.os == std::env::consts::OS
    }
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
        if !self.is_os() {
            return Ok(None);
        }
        builder::build(state, cmd_args)
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
