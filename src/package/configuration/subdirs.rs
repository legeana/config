use crate::package::configuration::parser;
use crate::package::configuration::util::no_args;
use crate::package::configuration::Configuration;

use anyhow::{anyhow, Context};

pub struct SubdirsParser;

const COMMAND: &str = "subdirs";

impl parser::Parser for SubdirsParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "subdirs
           load all subdirectories recursively"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        no_args(COMMAND, args)?;
        for entry in configuration
            .root
            .read_dir()
            .with_context(|| format!("failed to read {}", configuration.root.display()))?
        {
            let entry = entry
                .with_context(|| format!("failed to read {}", configuration.root.display()))?;
            let md = std::fs::metadata(entry.path()).with_context(|| {
                format!("failed to read metadata for {}", entry.path().display())
            })?;
            if !md.is_dir() {
                continue;
            }
            let fname = entry.file_name();
            let subdir = fname
                .to_str()
                .ok_or_else(|| anyhow!("failed to parse {:?}", fname))?;
            let subroot = entry.path();
            let mut substate = parser::State {
                prefix: state.prefix.join(subdir),
            };
            let subconf = Configuration::new_sub(&mut substate, subroot)?;
            if configuration.subdirs.contains_key(subdir) {
                return Err(anyhow!("{} already includes {}", configuration, subdir).into());
            }
            configuration.subdirs.insert(subdir.to_owned(), subconf);
        }
        Ok(())
    }
}
