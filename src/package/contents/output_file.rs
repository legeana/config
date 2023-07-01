use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::ast;
use super::inventory;
use super::local_state;
use super::util;

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

impl ast::Statement for OutputFileStatement {
    fn eval(&self, state: &mut ast::State) -> Result<Option<ModuleBox>> {
        let dst = state.dst_path(&self.filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(OutputFile { output })))
    }
}

#[derive(Clone)]
struct OutputFileParser;

impl ast::Parser for OutputFileParser {
    fn name(&self) -> String {
        "output_file".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a symlink for filename in prefix to a local persistent state
        ", command=self.name()}
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<ast::StatementBox> {
        let filename = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(OutputFileStatement { filename }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_parser(Box::new(OutputFileParser {}));
}
