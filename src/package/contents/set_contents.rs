use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::local_state;
use super::util;

struct SetContents {
    output: local_state::FileState,
    contents: String,
}

impl Module for SetContents {
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(rules, registry)?;
        let state = self.output.path();
        if state
            .try_exists()
            .with_context(|| format!("unable to check if {state:?} exists"))?
        {
            log::info!("Copy: skipping already existing state for {state:?}");
            return Ok(());
        }
        std::fs::write(state, &self.contents)
            .with_context(|| format!("unable to write {:?} to {:?}", self.contents, state))?;
        Ok(())
    }
}

#[derive(Debug)]
struct SetContentsBuilder {
    filename: String,
    contents: String,
}

impl builder::Builder for SetContentsBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let dst = state.dst_path(&self.filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(SetContents {
            output,
            contents: self.contents.clone(),
        })))
    }
}

#[derive(Clone)]
struct SetContentsParser;

impl builder::Parser for SetContentsParser {
    fn name(&self) -> String {
        "set_contents".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename> <contents>
                overwrites <filename> with <contents>
        ", command=self.name()}
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let (filename, contents) = util::double_arg(&self.name(), args)?;
        Ok(Box::new(SetContentsBuilder {
            filename: filename.to_owned(),
            contents: contents.to_owned(),
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(SetContentsParser {})]
}
