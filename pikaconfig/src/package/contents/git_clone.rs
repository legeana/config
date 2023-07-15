use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::git_utils;
use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::engine;
use super::inventory;
use super::local_state;
use super::util;

struct GitClone {
    remote: git_utils::Remote,
    output: local_state::DirectoryState,
}

impl GitClone {
    fn need_update(&self) -> Result<bool> {
        let root = self.output.path();
        let remote_url = git_utils::get_remote_url(root)?;
        if remote_url != self.remote.url {
            return Ok(true);
        }
        let target_branch = match self.remote.branch {
            Some(ref branch) => branch.clone(),
            None => git_utils::get_remote_head_ref(root)?,
        };
        let current_branch = git_utils::get_head_ref(root)?;
        Ok(target_branch != current_branch)
    }
    fn force_pull(&self) -> Result<()> {
        git_utils::git_force_shallow_pull(self.output.path(), &self.remote)
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
            if !self.need_update()? && !rules.force_download {
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

impl engine::Statement for GitCloneStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let dst = ctx.dst_path(&self.dst);
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

impl engine::Parser for GitCloneParser {
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
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<engine::StatementBox> {
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
