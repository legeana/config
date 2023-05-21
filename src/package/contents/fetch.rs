use anyhow::{self, Context, Result};

use super::local_state;
use super::parser;
use super::util;
use crate::registry::Registry;

pub struct FetchIntoParser {}

const COMMAND: &str = "fetch_into";

struct FetchIntoInstaller {
    url: String,
    output: local_state::FileState,
}

impl super::Module for FetchIntoInstaller {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(registry)?;
        let state = self.output.path();
        if state
            .try_exists()
            .with_context(|| format!("unable to check if {state:?} exists"))?
        {
            log::info!("Copy: skipping already existing state for {state:?}");
            return Ok(());
        }
        let mut reader = ureq::get(&self.url)
            .call()
            .with_context(|| format!("failed to fetch {:?}", self.url))?
            .into_reader();
        let output =
            std::fs::File::create(&state).with_context(|| format!("failed to open {state:?}"))?;
        let mut writer = std::io::BufWriter::new(output);
        std::io::copy(&mut reader, &mut writer)
            .with_context(|| format!("failed to write {state:?}"))?;
        Ok(())
    }
}

impl parser::Parser for FetchIntoParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "fetch_into <url> <filename>
           downloads <url> into a local storage and installs a symlink to it"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let args = util::fixed_args(COMMAND, args, 2)?;
        assert_eq!(args.len(), 2);
        let filename = args[0];
        let url = args[1];
        let dst = state.prefix.current.join(filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        configuration.modules.push(Box::new(FetchIntoInstaller {
            url: url.to_owned(),
            output,
        }));
        Ok(())
    }
}
