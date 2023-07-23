use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::args::Arguments;
use super::engine;
use super::inventory;
use super::local_state;

struct OutputFile {
    output: local_state::FileState,
}

impl Module for OutputFile {
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(rules, registry)
    }
}

#[derive(Debug)]
struct OutputFileStatement {
    filename: String,
}

impl engine::Statement for OutputFileStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let dst = ctx.dst_path(&self.filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(OutputFile { output })))
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
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::StatementBox> {
        let filename = args.expect_single_arg(self.name())?.to_owned();
        Ok(Box::new(OutputFileStatement { filename }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(OutputFileBuilder {}));
}
