use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::ModuleBox;

use super::args::Arguments;
use super::engine;
use super::inventory;
use super::util;

#[derive(Debug)]
struct SubdirStatement {
    subdir: PathBuf,
    config: engine::StatementBox,
}

impl engine::Statement for SubdirStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let mut substate = engine::Context {
            enabled: true,
            prefix: ctx.prefix.join(&self.subdir),
        };
        self.config.eval(&mut substate)
    }
}

#[derive(Clone)]
struct SubdirBuilder;

impl engine::CommandBuilder for SubdirBuilder {
    fn name(&self) -> String {
        "subdir".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <subdirectory>
                load subdirectory configuration recursively
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::StatementBox> {
        let subdir = util::single_arg(&self.name(), args)?;
        let subroot = workdir.join(subdir);
        Ok(Box::new(SubdirStatement {
            subdir: subdir.into(),
            config: super::ConfigurationStatement::parse(subroot)?,
        }))
    }
}

#[derive(Debug)]
struct SubdirsStatement {
    subdirs: Vec<SubdirStatement>,
}

impl engine::Statement for SubdirsStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let mut modules: Vec<ModuleBox> = Vec::new();
        for subdir in self.subdirs.iter() {
            if let Some(m) = subdir.eval(ctx)? {
                modules.push(m);
            }
        }
        Ok(Some(Box::new(modules)))
    }
}

#[derive(Clone)]
struct SubdirsBuilder;

impl engine::CommandBuilder for SubdirsBuilder {
    fn name(&self) -> String {
        "subdirs".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command}
                load all subdirectories recursively
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::StatementBox> {
        util::no_args(&self.name(), args)?;
        let mut subdirs: Vec<SubdirStatement> = Vec::new();
        for entry in workdir
            .read_dir()
            .with_context(|| format!("failed to read {:?}", workdir))?
        {
            let entry = entry.with_context(|| format!("failed to read {:?}", workdir))?;
            let md = std::fs::metadata(entry.path())
                .with_context(|| format!("failed to read metadata for {:?}", entry.path()))?;
            if !md.is_dir() {
                continue;
            }
            let fname = entry.file_name();
            subdirs.push(SubdirStatement {
                subdir: fname.into(),
                config: super::ConfigurationStatement::parse(entry.path())?,
            });
        }
        Ok(Box::new(SubdirsStatement { subdirs }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(SubdirBuilder {}));
    registry.register_command(Box::new(SubdirsBuilder {}));
}
