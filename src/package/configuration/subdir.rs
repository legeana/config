use crate::package::configuration::parser::{Error, Parser, Result};
use crate::package::configuration::util::single_arg;
use crate::package::configuration::Configuration;

use anyhow::anyhow;

pub struct SubdirParser {}

const COMMAND: &str = "subdir";

impl Parser for SubdirParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "subdir <subdirectory>
           load subdirectory configuration recursively"
    }
    fn parse(&self, configuration: &mut Configuration, args: &[&str]) -> Result<()> {
        let subdir = single_arg(COMMAND, args)?;
        let subroot = configuration.root.clone().join(subdir);
        let subconf = Configuration::new(subroot)?;
        // TODO: use try_insert when available
        if configuration.subdirs.contains_key(subdir) {
            return Err(Error::Other(anyhow!(
                "{} already includes {}",
                configuration,
                subdir
            )));
        }
        configuration.subdirs.insert(subdir.to_string(), subconf);
        return Ok(());
    }
}
