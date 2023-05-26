use std::path::PathBuf;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::local_state;
use super::util;

use anyhow::{Context, Result};

pub struct CopyBuilder;

const COMMAND: &str = "copy";

struct Copy {
    src: PathBuf,
    output: local_state::FileState,
}

impl Module for Copy {
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
        std::fs::copy(&self.src, state)
            .with_context(|| format!("unable to copy {:?} to {state:?}", self.src))?;
        Ok(())
    }
}

impl builder::Builder for CopyBuilder {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "copy <filename>
           create a copy of a filename in local storage and install a symlink to it"
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let filename = util::single_arg(COMMAND, args)?;
        let dst = state.prefix.dst_path(filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        Ok(Some(Box::new(Copy {
            src: state.prefix.src_path(filename),
            output,
        })))
    }
}
