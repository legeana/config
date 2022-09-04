use std::path::PathBuf;

use crate::package::configuration::file_util::make_symlink;
use crate::package::configuration::parser;
use crate::package::configuration::util::single_arg;
use crate::package::configuration::Configuration;
use crate::registry::Registry;

use anyhow::{self, Context};
use walkdir::WalkDir;

pub struct SymlinkTreeParser {}

const COMMAND: &str = "symlink_tree";

struct SymlinkTreeInstaller {
    src: PathBuf,
    dst: PathBuf,
}

impl super::FileInstaller for SymlinkTreeInstaller {
    fn install(&self, registry: &mut dyn Registry) -> anyhow::Result<()> {
        for e in WalkDir::new(&self.src).sort_by_file_name() {
            let entry = e.with_context(|| format!("failed to read {}", self.src.display()))?;
            let src = entry.path();
            let suffix = src.strip_prefix(&self.src).with_context(|| {
                format!(
                    "unable to remove prefix {} from {}",
                    self.src.display(),
                    src.display()
                )
            })?;
            let dst = self.dst.join(suffix);
            make_symlink(&src, &dst)?;
            registry.register_symlink(&dst)?;
        }
        return Ok(());
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
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let filename = single_arg(COMMAND, args)?;
        configuration.files.push(Box::new(SymlinkTreeInstaller {
            src: configuration.root.join(filename),
            dst: state.prefix.current.join(filename),
        }));
        return Ok(());
    }
}
