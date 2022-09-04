use std::path::PathBuf;

use crate::package::configuration::file_util::make_local_state;
use crate::package::configuration::parser;
use crate::package::configuration::util::single_arg;
use crate::package::configuration::Configuration;
use crate::registry::Registry;

use anyhow::{self, Context};

pub struct CopyParser {}

const COMMAND: &str = "copy";

struct CopyInstaller {
    src: PathBuf,
    dst: PathBuf,
}

impl super::FileInstaller for CopyInstaller {
    fn install(&self, registry: &mut dyn Registry) -> anyhow::Result<()> {
        let state = make_local_state(&self.dst)?;
        registry.register_symlink(&self.dst)?;
        std::fs::copy(&self.src, &state).with_context(|| {
            format!(
                "unable to copy {} to {}",
                self.src.display(),
                state.display()
            )
        })?;
        return Ok(());
    }
}

impl parser::Parser for CopyParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "copy filename
           create a copy of a filename in local storage and install a symlink to it"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let filename = single_arg(COMMAND, args)?;
        configuration.files.push(Box::new(CopyInstaller {
            src: configuration.root.join(filename),
            dst: state.prefix.current.join(filename),
        }));
        return Ok(());
    }
}
