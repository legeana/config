use anyhow::{Context, Result};

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::local_state;
use super::parser;
use super::util;

pub struct FetchIntoParser {}

const COMMAND: &str = "fetch_into";

struct FetchInto {
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
            return Ok(());
        }
        let mut reader = ureq::get(&self.url)
            .call()
            .with_context(|| format!("failed to fetch {:?}", self.url))?
            .into_reader();
        let output =
            std::fs::File::create(state).with_context(|| format!("failed to open {state:?}"))?;
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
        "fetch_into <filename> <url>
           downloads <url> into a local storage and installs a symlink to it"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        args: &[&str],
    ) -> Result<Option<Box<dyn Module>>> {
        let args = util::fixed_args(COMMAND, args, 2)?;
        assert_eq!(args.len(), 2);
        let filename = args[0];
        let url = args[1];
        let dst = state.prefix.dst_path(filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        Ok(Some(Box::new(FetchInto {
            url: url.to_owned(),
            output,
        })))
    }
}
