use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::local_state;
use super::util;

pub struct SetContentsBuilder;

const COMMAND: &str = "set_contents";

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

impl builder::Builder for SetContentsBuilder {
    fn name(&self) -> String {
        COMMAND.to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {COMMAND} <filename> <contents>
                overwrites <filename> with <contents>
        "}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let args = util::fixed_args(COMMAND, args, 2)?;
        assert_eq!(args.len(), 2);
        let filename = args[0];
        let contents = args[1];
        let dst = state.prefix.dst_path(filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(SetContents {
            output,
            contents: contents.to_owned(),
        })))
    }
}
