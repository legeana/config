use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::local_state;
use super::util;

struct FetchInto {
    executable: bool,
    url: String,
    output: local_state::FileState,
}

impl FetchInto {
    #[cfg(unix)]
    fn set_executable(&self, f: &std::fs::File) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
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

#[derive(Debug)]
struct FetchIntoBuilder {
    filename: String,
    url: String,
    executable: bool,
}

impl builder::Builder for FetchIntoBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let dst = state.dst_path(&self.filename);
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
struct FetchIntoParser;

impl builder::Parser for FetchIntoParser {
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
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let (filename, url) = util::double_arg(&self.name(), args)?;
        Ok(Box::new(FetchIntoBuilder {
            filename: filename.to_owned(),
            url: url.to_owned(),
            executable: false,
        }))
    }
}

#[derive(Clone)]
struct FetchExeIntoParser;

impl builder::Parser for FetchExeIntoParser {
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
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let (filename, url) = util::double_arg(&self.name(), args)?;
        Ok(Box::new(FetchIntoBuilder {
            filename: filename.to_owned(),
            url: url.to_owned(),
            executable: true,
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![
        Box::new(FetchIntoParser {}),
        Box::new(FetchExeIntoParser {}),
    ]
}
