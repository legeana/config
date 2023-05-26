use std::path::PathBuf;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::util;

use anyhow::{Context, Result};

pub struct MkDirBuilder;

const COMMAND: &str = "mkdir";

struct MkDir {
    dst: PathBuf,
}

impl Module for MkDir {
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        std::fs::create_dir_all(&self.dst)
            .with_context(|| format!("unable to create {:?}", self.dst))?;
        registry
            .register(&self.dst)
            .with_context(|| format!("failed to register directory {:?}", self.dst))?;
        Ok(())
    }
}

impl builder::Builder for MkDirBuilder {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "mkdir <directory>
           create a directory in prefix"
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let filename = util::single_arg(COMMAND, args)?;
        Ok(Some(Box::new(MkDir {
            dst: state.prefix.dst_path(filename),
        })))
    }
}
