use crate::package::configuration::parser;
use crate::package::configuration::util::single_arg;
use crate::package::configuration::Configuration;

use anyhow::anyhow;

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
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let subdir = single_arg(COMMAND, args)?;
        let subroot = configuration.root.clone().join(subdir);
        let subconf = Configuration::new_sub(state, subroot)?;
        // TODO: use try_insert when available
        if configuration.subdirs.contains_key(subdir) {
            return Err(parser::Error::Other(anyhow!(
                "{} already includes {}",
                configuration,
                subdir
            )));
        }
        configuration.subdirs.insert(subdir.to_string(), subconf);
        return Ok(());
    }
}
