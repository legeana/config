use super::parser;
use super::util;

use anyhow::{anyhow, Result};

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
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let subdir = util::single_arg(COMMAND, args)?;
        let subroot = configuration.root.clone().join(subdir);
        let mut substate = parser::State {
            prefix: state.prefix.join(subdir),
        };
        let subconf = super::Configuration::new_sub(&mut substate, subroot)?;
        // TODO: use try_insert when available
        if configuration.subdirs.contains_key(subdir) {
            return Err(anyhow!("{} already includes {}", configuration, subdir));
        }
        configuration.subdirs.insert(subdir.to_string(), subconf);
        Ok(())
    }
}
