use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::Module;

use super::builder;
use super::util;

fn make_subdir(
    state: &mut builder::State,
    workdir: &Path,
    subdir: &Path,
) -> Result<Box<dyn Module>> {
    let mut substate = builder::State {
        enabled: true,
        prefix: state.prefix.join(subdir),
    };
    let subroot = workdir.join(subdir);
    let subconf = super::Configuration::new_sub(&mut substate, subroot)?;
    Ok(Box::new(subconf))
}

#[derive(Debug)]
struct SubdirBuilder {
    workdir: PathBuf,
    subdir: String,
}

impl builder::Builder for SubdirBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        Ok(Some(make_subdir(
            state,
            &self.workdir,
            Path::new(&self.subdir),
        )?))
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
        let subdir = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(SubdirBuilder {
            workdir: workdir.to_owned(),
            subdir,
        }))
    }
}

#[derive(Debug)]
struct SubdirsBuilder {
    workdir: PathBuf,
}

impl builder::Builder for SubdirsBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let mut modules: Vec<Box<dyn Module>> = Vec::new();
        for entry in self
            .workdir
            .read_dir()
            .with_context(|| format!("failed to read {:?}", self.workdir))?
        {
            let entry = entry.with_context(|| format!("failed to read {:?}", self.workdir))?;
            let md = std::fs::metadata(entry.path())
                .with_context(|| format!("failed to read metadata for {:?}", entry.path()))?;
            if !md.is_dir() {
                continue;
            }
            let fname = entry.file_name();
            modules.push(make_subdir(state, &self.workdir, Path::new(&fname))?);
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
        Ok(Box::new(SubdirsBuilder {
            workdir: workdir.to_owned(),
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(SubdirParser {}), Box::new(SubdirsParser {})]
}
