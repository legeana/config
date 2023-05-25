use anyhow::{Context, Result};

use crate::git_utils;
use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::local_state;
use super::parser;
use super::util;

pub struct GitCloneParser {}

const COMMAND: &str = "git_clone";

struct GitClone {
    remote: git_utils::Remote,
    output: local_state::DirectoryState,
}

impl GitClone {
    fn force_pull(&self) -> Result<()> {
        git_utils::git_force_remote(self.output.path(), &self.remote)?;
        git_utils::git_force_shallow_pull(self.output.path())
    }
    fn clone(&self) -> Result<()> {
        git_utils::git_shallow_clone(&self.remote, self.output.path())
    }
    fn is_empty(&self) -> Result<bool> {
        let count = std::fs::read_dir(self.output.path())
            .with_context(|| format!("failed to read dir {:?}", self.output.path()))?
            .count();
        Ok(count == 0)
    }
}

impl Module for GitClone {
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(rules, registry)?;
        if self.is_empty()? {
            self.clone().with_context(|| {
                format!(
                    "failed to git clone {:?} for {:?}",
                    self.output.path(),
                    self.output.link()
                )
            })
        } else {
            if !rules.force_download {
                return Ok(());
            }
            self.force_pull().with_context(|| {
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
        "git_clone <url>[#<branch>] <directory>
           git clone <url> into a local storage and installs a symlink to it
           if <branch> is specified clone <branch> instead of default HEAD"
    }
    fn parse(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let args = util::fixed_args(COMMAND, args, 2)?;
        assert_eq!(args.len(), 2);
        let url = args[0];
        let filename = args[1];
        let dst = state.prefix.dst_path(filename);
        let output = local_state::DirectoryState::new(dst.clone())
            .with_context(|| format!("failed to create DirectoryState from {dst:?}"))?;
        Ok(Some(Box::new(GitClone {
            remote: git_utils::Remote::new(url),
            output,
        })))
    }
}
