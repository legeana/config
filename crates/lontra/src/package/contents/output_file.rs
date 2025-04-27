use std::path::Path;

use anyhow::{Context as _, Result};
use indoc::formatdoc;

use crate::module::BoxedModule;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

#[derive(Debug)]
struct OutputFileStatement {
    filename: Argument,
}

impl engine::Statement for OutputFileStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
        let dst = ctx.dst_path(ctx.expand_arg(&self.filename)?);
        let output = local_state::file_state(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(output)))
    }
}

#[derive(Clone)]
struct OutputFileBuilder;

impl engine::CommandBuilder for OutputFileBuilder {
    fn name(&self) -> String {
        "output_file".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a symlink for filename in prefix to a local persistent state
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let filename = args.expect_single_arg(self.name())?.clone();
        Ok(engine::Command::new_statement(OutputFileStatement {
            filename,
        }))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(OutputFileBuilder));
}
