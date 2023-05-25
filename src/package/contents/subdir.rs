use anyhow::Result;

use crate::module::Module;

use super::parser;
use super::util;

pub struct SubdirParser {}

const COMMAND: &str = "subdir";

impl parser::Parser for SubdirParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "subdir <subdirectory>
           load subdirectory configuration recursively"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        _configuration: &super::Configuration,
        args: &[&str],
    ) -> Result<Option<Box<dyn Module>>> {
        let subdir = util::single_arg(COMMAND, args)?;
        let mut substate = parser::State {
            enabled: true,
            prefix: state.prefix.join(subdir),
        };
        let subroot = substate.prefix.src_dir.clone();
        let subconf = super::Configuration::new_sub(&mut substate, subroot)?;
        Ok(Some(Box::new(subconf)))
    }
}
