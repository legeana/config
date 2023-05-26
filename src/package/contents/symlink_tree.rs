use std::path::PathBuf;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::file_util;
use super::parser;
use super::util;

use anyhow::{Context, Result};
use walkdir::WalkDir;

pub struct SymlinkTreeBuilder;

const COMMAND: &str = "symlink_tree";

struct SymlinkTree {
    src: PathBuf,
    dst: PathBuf,
}

impl Module for SymlinkTree {
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
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

impl parser::Builder for SymlinkTreeBuilder {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "symlink_tree <directory>
           create a symlink for every file in a directory recursively"
    }
    fn build(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let filename = util::single_arg(COMMAND, args)?;
        Ok(Some(Box::new(SymlinkTree {
            src: state.prefix.src_path(filename),
            dst: state.prefix.dst_path(filename),
        })))
    }
}
