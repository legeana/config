use std::path::{Path, PathBuf};

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::file_util;
use super::util;

use anyhow::{Context, Result};
use indoc::formatdoc;
use walkdir::WalkDir;

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

#[derive(Debug)]
struct SymlinkTreeStatement {
    workdir: PathBuf,
    directory: String,
}

impl builder::Statement for SymlinkTreeStatement {
    fn eval(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        Ok(Some(Box::new(SymlinkTree {
            src: self.workdir.join(&self.directory),
            dst: state.dst_path(&self.directory),
        })))
    }
}

#[derive(Clone)]
struct SymlinkTreeParser;

impl builder::Parser for SymlinkTreeParser {
    fn name(&self) -> String {
        "symlink_tree".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                create a symlink for every file in a directory recursively
        ", command=self.name()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Statement>> {
        let directory = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(SymlinkTreeStatement {
            workdir: workdir.to_owned(),
            directory,
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(SymlinkTreeParser {})]
}
