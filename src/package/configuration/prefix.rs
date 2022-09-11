use crate::package::configuration::parser;
use crate::package::configuration::util::single_arg;
use crate::package::configuration::Configuration;

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
        _configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let prefix = single_arg(COMMAND, args)?;
        state.prefix.set(shellexpand::tilde(prefix).as_ref().into());
        Ok(())
    }
}
