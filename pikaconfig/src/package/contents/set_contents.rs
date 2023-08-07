use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

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

#[derive(Debug)]
struct SetContentsStatement {
    filename: Argument,
    // This should be an Argument as well.
    // The challenge is writing OsString to file.
    // encode_wide() and as_bytes() can be used to achieve that.
    contents: String,
}

impl engine::Statement for SetContentsStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let dst = ctx.dst_path(ctx.expand_arg(&self.filename)?);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(SetContents {
            output,
            contents: self.contents.clone(),
        })))
    }
}

#[derive(Clone)]
struct SetContentsBuilder;

impl engine::CommandBuilder for SetContentsBuilder {
    fn name(&self) -> String {
        "set_contents".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename> <contents>
                overwrites <filename> with <contents>
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (filename, contents) = args.expect_double_arg(self.name())?;
        Ok(engine::Command::new_statement(SetContentsStatement {
            filename: filename.clone(),
            contents: contents.expect_raw().context("contents")?.to_owned(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(SetContentsBuilder {}));
}
