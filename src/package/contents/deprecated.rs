use anyhow::Result;

use crate::module::Module;

use super::builder;
use super::util::check_command;

pub struct DeprecatedBuilder;

const COMMAND: &str = "deprecated commands, do not use";

impl builder::Builder for DeprecatedBuilder {
    fn name(&self) -> String {
        COMMAND.to_owned()
    }
    fn help(&self) -> String {
        "DEPRECATED: N/A".to_owned()
    }
    fn build(&self, _state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
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
