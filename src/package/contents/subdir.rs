use anyhow::Result;

use crate::module::Module;

use super::builder;
use super::util;

pub struct SubdirBuilder;

const COMMAND: &str = "subdir";

impl builder::Builder for SubdirBuilder {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "subdir <subdirectory>
           load subdirectory configuration recursively"
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let subdir = util::single_arg(COMMAND, args)?;
        let mut substate = builder::State {
            enabled: true,
            prefix: state.prefix.join(subdir),
        };
        let subroot = substate.prefix.src_dir.clone();
        let subconf = super::Configuration::new_sub(&mut substate, subroot)?;
        Ok(Some(Box::new(subconf)))
    }
}
