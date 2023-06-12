#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::local_state;
use super::util;

pub struct FetchIntoBuilder;
pub struct FetchExeIntoBuilder;

struct FetchInto {
    executable: bool,
    url: String,
    output: local_state::FileState,
}

impl FetchInto {
    #[cfg(unix)]
    fn set_executable(&self, f: &std::fs::File) -> Result<()> {
        let metadata = f.metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        f.set_permissions(permissions)?;
        Ok(())
    }
    #[cfg(windows)]
    fn set_executable(&self, _f: &std::fs::File) -> Result<()> {
        // Nothing to do on Windows.
        Ok(())
    }
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
            self.set_executable(&output)
                .with_context(|| format!("failed to make {:?} executable", self.output.path()))?;
        }
        Ok(())
    }
}

fn build(
    command: &str,
    state: &mut builder::State,
    args: &[&str],
    executable: bool,
) -> Result<Option<Box<dyn Module>>> {
    let (filename, url) = util::double_arg(command, args)?;
    let dst = state.prefix.dst_path(filename);
    let output = local_state::FileState::new(dst.clone())
        .with_context(|| format!("failed to create FileState from {dst:?}"))?;
    Ok(Some(Box::new(FetchInto {
        executable,
        url: url.to_owned(),
        output,
    })))
}

impl builder::Builder for FetchIntoBuilder {
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
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        build(&self.name(), state, args, false)
    }
}

impl builder::Builder for FetchExeIntoBuilder {
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
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        build(&self.name(), state, args, true)
    }
}
