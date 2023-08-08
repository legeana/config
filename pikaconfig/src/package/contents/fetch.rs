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

struct FetchInto {
    executable: bool,
    url: String,
    output: local_state::FileState,
}

impl Module for FetchInto {
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(rules, registry)?;
        let state = self.output.path();
        if state
            .try_exists()
            .with_context(|| format!("unable to check if {state:?} exists"))?
        {
            log::info!("Copy: skipping already existing state for {state:?}");
            // TODO: set_executable if necessary
            return Ok(());
        }
        let mut reader = ureq::get(&self.url)
            .call()
            .with_context(|| format!("failed to fetch {:?}", self.url))?
            .into_reader();
        let output =
            std::fs::File::create(state).with_context(|| format!("failed to open {state:?}"))?;
        let mut writer = std::io::BufWriter::new(&output);
        std::io::copy(&mut reader, &mut writer)
            .with_context(|| format!("failed to write {state:?}"))?;
        if self.executable {
            file_util::set_file_executable(&output)
                .with_context(|| format!("failed to make {state:?} executable"))?;
        }
        output
            .sync_all()
            .with_context(|| format!("failed to flush {state:?}"))
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
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        Ok(Some(Box::new(FetchInto {
            executable: self.executable,
            url: self.url.clone(),
            output,
        })))
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
