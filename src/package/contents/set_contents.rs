use std::path::PathBuf;

use super::file_util;
use super::parser;
use super::util;
use crate::registry::Registry;

use anyhow::{self, Context, Result};

pub struct SetContentsParser {}

const COMMAND: &str = "set_contents";

struct SetContentsInstaller {
    dst: PathBuf,
    contents: String,
}

impl super::FileInstaller for SetContentsInstaller {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        let state = file_util::make_local_state(registry, &self.dst)?;
        if state
            .try_exists()
            .with_context(|| format!("unable to check if {state:?} exists"))?
        {
            log::info!("Copy: skipping already existing state for {state:?}");
            return Ok(());
        }
        std::fs::write(&self.dst, &self.contents)
            .with_context(|| format!("unable to write {:?} to {:?}", self.contents, self.dst))?;
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
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let args = util::fixed_args(COMMAND, args, 2)?;
        assert_eq!(args.len(), 2);
        let filename = args[0];
        let contents = args[1];
        configuration.files.push(Box::new(SetContentsInstaller {
            dst: state.prefix.current.join(filename),
            contents: contents.to_owned(),
        }));
        Ok(())
    }
}
