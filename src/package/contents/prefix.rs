use anyhow::Result;

use super::parser;
use super::util;

pub struct PrefixParser {}

const COMMAND: &str = "prefix";

impl parser::Parser for PrefixParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "prefix <directory>
           set current installation prefix to <directory>"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        _configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let prefix = util::single_arg(COMMAND, args)?;
        state.prefix.set(shellexpand::tilde(prefix).as_ref().into());
        Ok(())
    }
}
