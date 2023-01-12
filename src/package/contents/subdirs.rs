use super::parser;
use super::util;

use anyhow::{anyhow, Context, Result};

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
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        util::no_args(COMMAND, args)?;
        for entry in configuration
            .root
            .read_dir()
            .with_context(|| format!("failed to read {:?}", configuration.root))?
        {
            let entry =
                entry.with_context(|| format!("failed to read {:?}", configuration.root))?;
            let md = std::fs::metadata(entry.path())
                .with_context(|| format!("failed to read metadata for {:?}", entry.path()))?;
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
            let subconf = super::Configuration::new_sub(&mut substate, subroot)?;
            if configuration.subdirs.contains_key(subdir) {
                return Err(anyhow!("{configuration} already includes {subdir}"));
            }
            configuration.subdirs.insert(subdir.to_owned(), subconf);
        }
        Ok(())
    }
}
