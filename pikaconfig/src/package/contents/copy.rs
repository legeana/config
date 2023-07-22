use std::path::{Path, PathBuf};

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::engine;
use super::inventory;
use super::local_state;
use super::util;

use anyhow::{Context, Result};
use indoc::formatdoc;

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

#[derive(Debug)]
struct CopyStatement {
    workdir: PathBuf,
    filename: String,
}

impl engine::Statement for CopyStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let dst = ctx.dst_path(&self.filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        Ok(Some(Box::new(Copy {
            src: self.workdir.join(&self.filename),
            output,
        })))
    }
}

#[derive(Clone)]
struct CopyParser;

impl engine::Parser for CopyParser {
    fn name(&self) -> String {
        "copy".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a copy of a filename in local storage and install a symlink to it
        ", command=self.name()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<engine::StatementBox> {
        let filename = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(CopyStatement {
            workdir: workdir.to_owned(),
            filename,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(CopyParser {}));
}
