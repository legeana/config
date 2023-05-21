use std::path::PathBuf;

use super::local_state;
use super::parser;
use super::util;
use crate::registry::Registry;

use anyhow::{self, Context, Result};

pub struct CopyParser {}

const COMMAND: &str = "copy";

struct Copy {
    src: PathBuf,
    output: local_state::FileState,
}

impl super::Module for Copy {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(registry)?;
        let state = self.output.path();
        if state
            .try_exists()
            .with_context(|| format!("unable to check if {state:?} exists"))?
        {
            log::info!("Copy: skipping already existing state for {state:?}");
            return Ok(());
        }
        std::fs::copy(&self.src, state)
            .with_context(|| format!("unable to copy {:?} to {state:?}", self.src))?;
        Ok(())
    }
}

impl parser::Parser for CopyParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "copy <filename>
           create a copy of a filename in local storage and install a symlink to it"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let filename = util::single_arg(COMMAND, args)?;
        let dst = state.prefix.current.join(filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        configuration.modules.push(Box::new(Copy {
            src: configuration.root.join(filename),
            output,
        }));
        Ok(())
    }
}
