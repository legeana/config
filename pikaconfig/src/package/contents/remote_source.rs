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
            && !rules.force_download
            && self
                .archive
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
        log::info!("Fetch: {:?} -> {:?}", self.url, self.archive);
        // TODO: Use checksum to verify version/integrity.
        net_util::fetch(&self.url, &self.archive, &net_util::FetchOptions::new())
            .with_context(|| format!("failed to fetch {:?}", self.url))?;
        Ok(true)
    }
    fn unpack(&self, rules: &Rules, force: bool) -> Result<bool> {
        if !force
            && !rules.force_download
            && !is_dir_empty(&self.source)
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
            .unarchive(&self.archive, &self.source)
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
    url: String,
}

impl engine::Expression for RemoteArchiveExpression {
    fn eval(&self, _ctx: &mut engine::Context) -> Result<engine::ExpressionOutput> {
        let archive =
            local_state::EphemeralFileCache::new(&self.workdir, Path::new(&self.filename))?;
        let archive_path = archive.path().to_owned();
        let source = local_state::EphemeralDirCache::new(&self.workdir, Path::new(&self.filename))?;
        let source_path = source.path().to_owned();
        // TODO: consider building this in Builder.
        // This will not evaluate in a false branch of an if statement.
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
        let filename = filename.expect_raw().context("filename")?.to_owned();
        let url = url.expect_raw().context("url")?.to_owned();
        url::Url::parse(&url)
            .with_context(|| format!("failed to parse URL {url:?}"))
            .context("URL verification")?;
        // TODO: add checksum.
        Ok(engine::Command::new_expression(RemoteArchiveExpression {
            workdir: workdir.to_owned(),
            filename,
            url,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(RemoteArchiveBuilder));
}
