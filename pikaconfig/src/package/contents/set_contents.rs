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
    output: local_state::StateBox,
    contents: String,
}

impl Module for SetContents {
    fn install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        if self
            .output
            .path()
            .try_exists()
            .with_context(|| format!("unable to check if {:?} exists", self.output))?
        {
            log::info!(
                "Copy: skipping already existing state for {:?}",
                self.output
            );
            return Ok(());
        }
        std::fs::write(self.output.path(), &self.contents)
            .with_context(|| format!("unable to write {:?} to {:?}", self.contents, self.output))?;
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
        let output = local_state::file_state(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        let output_state = output.state();
        Ok(Some(Box::new((
            output,
            SetContents {
                output: output_state,
                contents: self.contents.clone(),
            },
        ))))
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
