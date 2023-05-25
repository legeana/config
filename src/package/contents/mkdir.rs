use std::path::PathBuf;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::parser;
use super::util;

use anyhow::{Context, Result};

pub struct MkDirParser {}

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

impl parser::Parser for MkDirParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "mkdir <directory>
           create a directory in prefix"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        _configuration: &super::Configuration,
        args: &[&str],
    ) -> Result<Option<Box<dyn Module>>> {
        let filename = util::single_arg(COMMAND, args)?;
        Ok(Some(Box::new(MkDir {
            dst: state.prefix.current.join(filename),
        })))
    }
}
