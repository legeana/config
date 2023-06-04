use anyhow::Result;

use crate::module::Module;

use super::builder;
use super::util;

pub struct PrefixBuilder;

const COMMAND: &str = "prefix";

impl builder::Builder for PrefixBuilder {
    fn name(&self) -> String {
        COMMAND.to_owned()
    }
    fn help(&self) -> String {
        format!("{COMMAND} <directory>
           set current installation prefix to <directory>")
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let prefix = util::single_arg(COMMAND, args)?;
        state.prefix.set(shellexpand::tilde(prefix).as_ref().into());
        Ok(None)
    }
}
