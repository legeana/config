use anyhow::{anyhow, Context, Result};

use crate::module::Module;

use super::builder;
use super::util;

pub struct SubdirsBuilder;

const COMMAND: &str = "subdirs";

impl builder::Builder for SubdirsBuilder {
    fn name(&self) -> String {
        COMMAND.to_owned()
    }
    fn help(&self) -> String {
        format!("{COMMAND}
           load all subdirectories recursively")
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
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
            let mut substate = builder::State {
                enabled: true,
                prefix: state.prefix.join(subdir),
            };
            let subconf = super::Configuration::new_sub(&mut substate, subroot)?;
            modules.push(Box::new(subconf));
        }
        Ok(Some(Box::new(modules)))
    }
}
