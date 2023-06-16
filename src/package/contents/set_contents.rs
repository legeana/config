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
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (filename, contents) = util::double_arg(&self.name(), args)?;
        let dst = state.prefix.dst_path(filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(SetContents {
            output,
            contents: contents.to_owned(),
        })))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(SetContentsParser {})]
}
