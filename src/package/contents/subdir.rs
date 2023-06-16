use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::Module;

use super::builder;
use super::util;

#[derive(Debug)]
struct SubdirBuilder {
    subdir: PathBuf,
    config: Box<dyn builder::Builder>,
}

impl builder::Builder for SubdirBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let mut substate = builder::State {
            enabled: true,
            prefix: state.prefix.join(&self.subdir),
        };
        self.config.build(&mut substate)
    }
}

#[derive(Clone)]
struct SubdirParser;

impl builder::Parser for SubdirParser {
    fn name(&self) -> String {
        "subdir".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <subdirectory>
                load subdirectory configuration recursively
        ", command=self.name()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let subdir = util::single_arg(&self.name(), args)?;
        let subroot = workdir.join(subdir);
        Ok(Box::new(SubdirBuilder {
            subdir: subdir.into(),
            config: super::ConfigurationBuilder::parse(subroot)?,
        }))
    }
}

#[derive(Debug)]
struct SubdirsBuilder {
    subdirs: Vec<SubdirBuilder>,
}

impl builder::Builder for SubdirsBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let mut modules: Vec<Box<dyn Module>> = Vec::new();
        for subdir in self.subdirs.iter() {
            if let Some(m) = subdir.build(state)? {
                modules.push(m);
            }
        }
        Ok(Some(Box::new(modules)))
    }
}

#[derive(Clone)]
struct SubdirsParser;

impl builder::Parser for SubdirsParser {
    fn name(&self) -> String {
        "subdirs".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command}
                load all subdirectories recursively
        ", command=self.name()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        util::no_args(&self.name(), args)?;
        let mut subdirs: Vec<SubdirBuilder> = Vec::new();
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
            subdirs.push(SubdirBuilder {
                subdir: fname.into(),
                config: super::ConfigurationBuilder::parse(entry.path())?,
            });
        }
        Ok(Box::new(SubdirsBuilder { subdirs }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(SubdirParser {}), Box::new(SubdirsParser {})]
}
