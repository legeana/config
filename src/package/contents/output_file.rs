use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::package::contents::util;
use crate::registry::Registry;

use super::builder;
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
struct OutputFileBuilder {
    filename: String,
}

impl builder::Builder for OutputFileBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let dst = state.prefix.dst_path(&self.filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(OutputFile { output })))
    }
}

#[derive(Clone)]
struct OutputFileParser;

impl builder::Parser for OutputFileParser {
    fn name(&self) -> String {
        "output_file".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a symlink for filename in prefix to a local persistent state
        ", command=self.name()}
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let filename = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(OutputFileBuilder { filename }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(OutputFileParser {})]
}
