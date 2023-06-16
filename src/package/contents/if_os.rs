use crate::module::Module;

use super::builder;
use super::util;

use anyhow::Result;
use indoc::formatdoc;

#[derive(Clone)]
struct IfOsParser {
    os: &'static str,
}

impl IfOsParser {
    fn is_os(&self) -> bool {
        // We don't have to check this in runtime as this never changes.
        // In fact, it's even beneficial to check this during build to support *Prefix.
        self.os == std::env::consts::FAMILY || self.os == std::env::consts::OS
    }
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
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (empty, cmd_args) = util::multiple_args(&self.command(), args, 0)?;
        assert!(empty.is_empty());
        if !self.is_os() {
            return Ok(None);
        }
        builder::build(state, cmd_args)
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
