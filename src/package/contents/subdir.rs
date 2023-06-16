use anyhow::{anyhow, Context, Result};
use indoc::formatdoc;

use crate::module::Module;

use super::builder;
use super::util;

struct SubdirBuilder;

impl builder::Builder for SubdirBuilder {
    fn name(&self) -> String {
        "subdir".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <subdirectory>
                load subdirectory configuration recursively
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let subdir = util::single_arg(&self.name(), args)?;
        let mut substate = builder::State {
            enabled: true,
            prefix: state.prefix.join(subdir),
        };
        let subroot = substate.prefix.src_dir.clone();
        let subconf = super::Configuration::new_sub(&mut substate, subroot)?;
        Ok(Some(Box::new(subconf)))
    }
}

struct SubdirsBuilder;

impl builder::Builder for SubdirsBuilder {
    fn name(&self) -> String {
        "subdirs".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command}
                load all subdirectories recursively
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        util::no_args(&self.name(), args)?;
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

pub fn commands() -> Vec<Box<dyn builder::Builder>> {
    vec![Box::new(SubdirBuilder {}), Box::new(SubdirsBuilder {})]
}
