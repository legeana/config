use anyhow::Result;

use crate::module::Module;

use super::builder;
use super::util::check_command;

pub struct DeprecatedBuilder;

impl builder::Builder for DeprecatedBuilder {
    fn name(&self) -> String {
        "deprecated commands, do not use".to_owned()
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
        check_command(&self.name(), args).map(|_| ())?;
        Ok(None)
    }
}
