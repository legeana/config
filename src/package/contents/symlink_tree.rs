use std::path::PathBuf;

use super::file_util;
use super::parser;
use super::util;
use crate::registry::Registry;

use anyhow::{Context, Result};
use walkdir::WalkDir;

pub struct SymlinkTreeParser {}

const COMMAND: &str = "symlink_tree";

struct SymlinkTree {
    src: PathBuf,
    dst: PathBuf,
}

impl super::Module for SymlinkTree {
    fn install(&self, _rules: &super::Rules, registry: &mut dyn Registry) -> Result<()> {
        for e in WalkDir::new(&self.src).sort_by_file_name() {
            let entry = e.with_context(|| format!("failed to read {:?}", self.src))?;
            if entry.file_type().is_dir() {
                // Skip directories. They will be automatically created by make_symlink().
                continue;
            }
            let src = entry.path();
            let suffix = src
                .strip_prefix(&self.src)
                .with_context(|| format!("unable to remove prefix {:?} from {src:?}", self.src))?;
            let dst = self.dst.join(suffix);
            file_util::make_symlink(registry, src, &dst)?;
        }
        Ok(())
    }
}

impl parser::Parser for SymlinkTreeParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "symlink_tree <directory>
           create a symlink for every file in a directory recursively"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let filename = util::single_arg(COMMAND, args)?;
        configuration.modules.push(Box::new(SymlinkTree {
            src: configuration.root.join(filename),
            dst: state.prefix.current.join(filename),
        }));
        Ok(())
    }
}
