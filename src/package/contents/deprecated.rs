use anyhow::Result;

use crate::module::Module;

use super::parser;
use super::util::check_command;

pub struct DeprecatedParser;

const COMMAND: &str = "deprecated commands, do not use";

impl parser::Parser for DeprecatedParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "DEPRECATED: N/A"
    }
    fn parse(
        &self,
        _state: &mut parser::State,
        _configuration: &super::Configuration,
        args: &[&str],
    ) -> Result<Option<Box<dyn Module>>> {
        /*if check_command("<deprecated>", args).is_ok() {
            log::warn!(
                "{:?}: <deprecated> is unsupported",
                configuration.root
            );
            return Ok(());
        }*/
        check_command(COMMAND, args).map(|_| ())?;
        Ok(None)
    }
}
