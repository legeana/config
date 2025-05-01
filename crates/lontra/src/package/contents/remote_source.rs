use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use indoc::formatdoc;
use lontra_registry::Registry;

use crate::annotated_path::BoxedAnnotatedPath;
use crate::module::{Module, Rules};

use super::args::Arguments;
use super::engine;
use super::inventory;
use super::local_state;
use super::net_util::{FetchOptions, Url, fetch};

struct RemoteArchive {
    url: Url,
    archive: BoxedAnnotatedPath,
    source: BoxedAnnotatedPath,
    unarchiver: &'static dyn lontra_unarchiver::Unarchiver,
}

fn is_dir_empty(path: &Path) -> Result<bool> {
    Ok(path
        .read_dir()
        .with_context(|| format!("failed to read {path:?}"))?
        .next()
        .is_none())
}

impl RemoteArchive {
    fn fetch(&self, rules: &Rules, force: bool) -> Result<bool> {
        if !force
            && !rules.force_update
            && self
                .archive
                .as_path()
                .try_exists()
                .with_context(|| format!("failed to check if {:?} exists", self.archive))?
        {
            log::info!(
                "Fetch: skipping {:?} for already existing {:?}",
                self.url,
                self.archive
            );
            return Ok(false);
        }
        // TODO: Use checksum to verify version/integrity.
        fetch(&self.url, &self.archive, &FetchOptions::new())
            .with_context(|| format!("failed to fetch {:?}", self.url))?;
        Ok(true)
    }
    fn unpack(&self, rules: &Rules, force: bool) -> Result<bool> {
        if !force
            && !rules.force_update
            && !is_dir_empty(self.source.as_path())
                .with_context(|| format!("failed to check if {:?} is empty", self.source))?
        {
            log::info!(
                "Unpack: skipping already existing {:?} -> {:?}",
                self.archive,
                self.source
            );
            return Ok(false);
        }
        log::info!("Unpack: {:?} -> {:?}", self.archive, self.source);
        self.unarchiver
            .unarchive(self.archive.as_path(), self.source.as_path())
            .with_context(|| {
                format!("failed to unpack {:?} into {:?}", self.archive, self.source)
            })?;
        Ok(true)
    }
}

impl Module for RemoteArchive {
    fn pre_install(&self, rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        let force = false;
        let force = self.fetch(rules, force)? || force;
        let _force = self.unpack(rules, force)? || force;
        Ok(())
    }
}

#[derive(Debug)]
struct RemoteArchiveExpression {
    workdir: PathBuf,
    filename: String,
    url: Url,
}

impl engine::Expression for RemoteArchiveExpression {
    fn eval(&self, _ctx: &mut engine::Context) -> Result<engine::ExpressionOutput> {
        let archive =
            local_state::file_cache(&self.workdir, Path::new(&self.filename), self.url.text())?;
        let archive_path = archive.state();
        let source =
            local_state::dir_cache(&self.workdir, Path::new(&self.filename), self.url.text())?;
        let source_path = source.state();
        let output = source_path.to_path_buf().into_os_string();
        // TODO: consider building this in Builder.
        // This will not evaluate in a false branch of an if statement.
        let unarchiver = lontra_unarchiver::by_filename(archive_path.as_path())
            .with_context(|| format!("failed to find unarchiver for {archive_path:?}"))?;
        Ok(engine::ExpressionOutput {
            module: Some(Box::new((
                archive,
                source,
                RemoteArchive {
                    url: self.url.clone(),
                    archive: archive_path,
                    source: source_path,
                    unarchiver,
                },
            ))),
            output,
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
            <directory> = $({command} <filename> <url>)
                Downloads and unpacks remote archive,
                returns a path to unpacked directory.
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (filename, url) = args.expect_double_arg(self.name())?;
        let filename = filename.expect_raw().context("filename")?.to_owned();
        let url = url.expect_raw().context("url")?;
        let url = Url::new(url).with_context(|| format!("failed to parse URL {url:?}"))?;
        // TODO: add checksum.
        Ok(engine::Command::new_expression(RemoteArchiveExpression {
            workdir: workdir.to_owned(),
            filename,
            url,
        }))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(RemoteArchiveBuilder));
}
