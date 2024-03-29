use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::args::{Argument, Arguments};
use super::engine;
use super::file_util;
use super::inventory;
use super::local_state;
use super::net_util;

struct FetchInto {
    executable: bool,
    url: String,
    output: local_state::StateBox,
}

impl Module for FetchInto {
    fn install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        if self
            .output
            .path()
            .try_exists()
            .with_context(|| format!("unable to check if {:?} exists", self.output))?
        {
            log::info!(
                "Fetch: skipping already existing state for {:?}",
                self.output
            );
            log::info!("Fetch: setting {:?} executable", self.output);
            file_util::set_path_executable(self.output.path())
                .with_context(|| format!("failed to make {:?} executable", self.output))?;
            return Ok(());
        }
        // TODO: maybe move logging into net_util::fetch(), similar to symlink.
        // The challenge is logging self.output's actual path.
        log::info!("Fetching {:?} -> {:?}", self.url, self.output);
        net_util::fetch(
            &self.url,
            self.output.path(),
            net_util::FetchOptions::new().executable(self.executable),
        )
        .with_context(|| format!("failed to fetch {:?}", self.url))
    }
}

#[derive(Debug)]
struct FetchIntoStatement {
    filename: Argument,
    url: String,
    executable: bool,
}

impl engine::Statement for FetchIntoStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let dst = ctx.dst_path(ctx.expand_arg(&self.filename)?);
        let output = local_state::linked_file_cache(dst.clone(), &self.url)
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        let output_state = output.state();
        Ok(Some(Box::new((
            output,
            FetchInto {
                executable: self.executable,
                url: self.url.clone(),
                output: output_state,
            },
        ))))
    }
}

#[derive(Clone)]
struct FetchIntoBuilder;

impl engine::CommandBuilder for FetchIntoBuilder {
    fn name(&self) -> String {
        "fetch_into".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename> <url>
                downloads <url> into a local storage
                and installs a symlink to it
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (filename, url) = args.expect_double_arg(self.name())?;
        Ok(engine::Command::new_statement(FetchIntoStatement {
            filename: filename.clone(),
            url: url.expect_raw().context("url")?.to_owned(),
            executable: false,
        }))
    }
}

#[derive(Clone)]
struct FetchExeIntoBuilder;

impl engine::CommandBuilder for FetchExeIntoBuilder {
    fn name(&self) -> String {
        "fetch_exe_into".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename> <url>
                downloads <url> into a local storage (with executable bit)
                and installs a symlink to it
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (filename, url) = args.expect_double_arg(self.name())?;
        Ok(engine::Command::new_statement(FetchIntoStatement {
            filename: filename.clone(),
            url: url.expect_raw().context("url")?.to_owned(),
            executable: true,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(FetchIntoBuilder));
    registry.register_command(Box::new(FetchExeIntoBuilder));
}
