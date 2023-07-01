use std::path::{Path, PathBuf};

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::builder;
use super::inventory;
use super::util;

use anyhow::{Context, Result};
use indoc::formatdoc;

struct MkDir {
    dst: PathBuf,
}

impl Module for MkDir {
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        std::fs::create_dir_all(&self.dst)
            .with_context(|| format!("unable to create {:?}", self.dst))?;
        registry
            .register(&self.dst)
            .with_context(|| format!("failed to register directory {:?}", self.dst))?;
        Ok(())
    }
}

#[derive(Debug)]
struct MkDirStatement {
    dir: String,
}

impl builder::Statement for MkDirStatement {
    fn eval(&self, state: &mut builder::State) -> Result<Option<ModuleBox>> {
        Ok(Some(Box::new(MkDir {
            dst: state.dst_path(&self.dir),
        })))
    }
}

#[derive(Clone)]
struct MkDirParser;

impl builder::Parser for MkDirParser {
    fn name(&self) -> String {
        "mkdir".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                create a directory in prefix
        ", command=self.name()}
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<builder::StatementBox> {
        let dir = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(MkDirStatement { dir }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_parser(Box::new(MkDirParser {}));
}
