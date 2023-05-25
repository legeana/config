use anyhow::{Context, Result};

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::local_state;
use super::parser;
use super::util;

pub struct SetContentsParser {}

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

impl parser::Parser for SetContentsParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "set_contents <filename> <contents>
           overwrites <filename> with <contents>"
    }
    fn parse(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
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
