use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::registry::Registry;
use crate::unarchiver;

use super::args::Arguments;
use super::engine;
use super::inventory;
use super::local_state;
use super::net_util;

struct RemoteArchive {
    url: String,
    archive: PathBuf,
    source: PathBuf,
    unarchiver: &'static dyn unarchiver::Unarchiver,
}

impl Module for RemoteArchive {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        net_util::fetch(&self.url, &self.archive, &net_util::FetchOptions::new())
            .with_context(|| format!("failed to fetch {:?}", self.url))?;
        self.unarchiver
            .unarchive(&self.archive, &self.source)
            .with_context(|| {
                format!("failed to unpack {:?} into {:?}", self.archive, self.source)
            })?;
        Ok(())
    }
}

#[derive(Debug)]
struct RemoteArchiveExpression {
    workdir: PathBuf,
    filename: String,
    url: String,
}

impl engine::Expression for RemoteArchiveExpression {
    fn eval(&self, _ctx: &mut engine::Context) -> Result<engine::ExpressionOutput> {
        let archive =
            local_state::EphemeralFileState::new(&self.workdir, Path::new(&self.filename))?;
        let archive_path = archive.path().to_owned();
        let source = local_state::EphemeralDirState::new(&self.workdir, Path::new(&self.filename))?;
        let source_path = source.path().to_owned();
        let unarchiver = unarchiver::by_filename(&archive_path)
            .with_context(|| format!("failed to find unarchiver for {archive_path:?}"))?;
        Ok(engine::ExpressionOutput {
            module: Some(Box::new((
                archive,
                source,
                RemoteArchive {
                    url: self.url.clone(),
                    archive: archive_path,
                    source: source_path.clone(),
                    unarchiver,
                },
            ))),
            output: source_path.into_os_string(),
        })
    }
}

#[derive(Clone)]
struct RemoteArchiveBuilder;

impl engine::CommandBuilder for RemoteArchiveBuilder {
    fn name(&self) -> String {
        "remote_archive".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            <directory> = {command} <filename> <url>
                Downloads and unpacks remote archive,
                returns a path to unpacked directory.
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (filename, url) = args.expect_double_arg(self.name())?;
        Ok(engine::Command::new_expression(RemoteArchiveExpression {
            workdir: workdir.to_owned(),
            filename: filename.expect_raw().context("filename")?.to_owned(),
            url: url.expect_raw().context("url")?.to_owned(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(RemoteArchiveBuilder));
}
