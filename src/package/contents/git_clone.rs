use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::git_utils;
use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::ast;
use super::inventory;
use super::local_state;
use super::util;

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

#[derive(Debug)]
struct GitCloneStatement {
    url: String,
    dst: String,
}

impl ast::Statement for GitCloneStatement {
    fn eval(&self, state: &mut ast::State) -> Result<Option<ModuleBox>> {
        let dst = state.dst_path(&self.dst);
        let output = local_state::DirectoryState::new(dst.clone())
            .with_context(|| format!("failed to create DirectoryState from {dst:?}"))?;
        Ok(Some(Box::new(GitClone {
            remote: git_utils::Remote::new(&self.url),
            output,
        })))
    }
}

#[derive(Clone)]
struct GitCloneParser;

impl ast::Parser for GitCloneParser {
    fn name(&self) -> String {
        "git_clone".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <url>[#<branch>] <directory>
                git clone <url> into a local storage and installs a symlink to it
                if <branch> is specified clone <branch> instead of default HEAD
        ", command=self.name()}
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<ast::StatementBox> {
        let (url, dst) = util::double_arg(&self.name(), args)?;
        Ok(Box::new(GitCloneStatement {
            url: url.to_owned(),
            dst: dst.to_owned(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_parser(Box::new(GitCloneParser {}));
}
