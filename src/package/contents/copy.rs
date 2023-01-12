use std::path::PathBuf;

use super::file_util;
use super::parser;
use super::util;
use crate::registry::Registry;

use anyhow::{self, Context, Result};

pub struct CopyParser {}

const COMMAND: &str = "copy";

struct CopyInstaller {
    src: PathBuf,
    dst: PathBuf,
}

impl super::FileInstaller for CopyInstaller {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        let state = file_util::make_local_state(registry, &self.dst)?;
        std::fs::copy(&self.src, &state)
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
        configuration.files.push(Box::new(CopyInstaller {
            src: configuration.root.join(filename),
            dst: state.prefix.current.join(filename),
        }));
        Ok(())
    }
}
