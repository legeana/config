use anyhow::{Context, Result};

use super::local_state;
use super::parser;
use super::util;
use crate::git_utils;
use crate::registry::Registry;

pub struct GitCloneParser {}

const COMMAND: &str = "git_clone";

struct GitClone {
    url: String,
    output: local_state::DirectoryState,
}

impl GitClone {
    fn hard_pull(&self) -> Result<()> {
        git_utils::git_hard_pull(self.output.path())
    }
    fn clone(&self) -> Result<()> {
        git_utils::git_clone(&self.url, self.output.path())
    }
    fn is_empty(&self) -> Result<bool> {
        let count = std::fs::read_dir(self.output.path())
            .with_context(|| format!("failed to read dir {:?}", self.output.path()))?
            .count();
        Ok(count == 0)
    }
}

impl super::Module for GitClone {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(registry)?;
        if self.is_empty()? {
            self.clone().with_context(|| {
                format!(
                    "failed to git clone {:?} for {:?}",
                    self.output.path(),
                    self.output.link()
                )
            })
        } else {
            self.hard_pull().with_context(|| {
                format!(
                    "failed to git pull {:?} for {:?}",
                    self.output.path(),
                    self.output.link()
                )
            })
        }
    }
}

impl parser::Parser for GitCloneParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "git_clone <url> <directory>
           git clone <url> into a local storage and installs a symlink to it"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let args = util::fixed_args(COMMAND, args, 2)?;
        assert_eq!(args.len(), 2);
        let url = args[0];
        let filename = args[1];
        let dst = state.prefix.current.join(filename);
        let output = local_state::DirectoryState::new(dst.clone())
            .with_context(|| format!("failed to create DirectoryState from {dst:?}"))?;
        configuration.modules.push(Box::new(GitClone {
            url: url.to_owned(),
            output,
        }));
        Ok(())
    }
}
