use anyhow::{anyhow, Context, Result};

use crate::module::Module;

use super::parser;
use super::util;

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
    fn parse(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        util::no_args(COMMAND, args)?;
        let mut modules: Vec<Box<dyn Module>> = Vec::new();
        for entry in state
            .prefix
            .src_dir
            .read_dir()
            .with_context(|| format!("failed to read {:?}", state.prefix.src_dir))?
        {
            let entry =
                entry.with_context(|| format!("failed to read {:?}", state.prefix.src_dir))?;
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
                enabled: true,
                prefix: state.prefix.join(subdir),
            };
            let subconf = super::Configuration::new_sub(&mut substate, subroot)?;
            modules.push(Box::new(subconf));
        }
        Ok(Some(Box::new(modules)))
    }
}
