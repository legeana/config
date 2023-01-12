use anyhow::Result;

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
        _configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        /*if check_command("<deprecated>", args).is_ok() {
            log::warn!(
                "{:?}: <deprecated> is unsupported",
                configuration.root
            );
            return Ok(());
        }*/
        return check_command(COMMAND, args).map(|_| ());
    }
}
