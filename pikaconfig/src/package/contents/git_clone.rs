use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::git_utils;
use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

struct GitClone {
    remote: git_utils::Remote,
    root: PathBuf,
    link: PathBuf,
}

impl GitClone {
    fn need_update(&self) -> Result<bool> {
        let remote_url = git_utils::get_remote_url(&self.root)?;
        if remote_url != self.remote.url {
            return Ok(true);
        }
        let target_branch = match self.remote.branch {
            Some(ref branch) => branch.clone(),
            None => git_utils::get_remote_head_ref(&self.root)?,
        };
        let current_branch = git_utils::get_head_ref(&self.root)?;
        Ok(target_branch != current_branch)
    }
    fn force_pull(&self) -> Result<()> {
        git_utils::git_force_shallow_pull(&self.root, &self.remote)
    }
    fn clone(&self) -> Result<()> {
        git_utils::git_shallow_clone(&self.remote, &self.root)
    }
    fn is_empty(&self) -> Result<bool> {
        let count = std::fs::read_dir(&self.root)
            .with_context(|| format!("failed to read dir {:?}", self.root))?
            .count();
        Ok(count == 0)
    }
}

impl Module for GitClone {
    fn install(&self, rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        if self.is_empty()? {
            self.clone().with_context(|| {
                format!("failed to git clone {:?} for {:?}", self.root, self.link,)
            })
        } else {
            if !self.need_update()? && !rules.force_download {
                return Ok(());
            }
            self.force_pull()
                .with_context(|| format!("failed to git pull {:?} for {:?}", self.root, self.link,))
        }
    }
}

#[derive(Debug)]
struct GitCloneStatement {
    url: String,
    dst: Argument,
}

impl engine::Statement for GitCloneStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let dst = ctx.dst_path(ctx.expand_arg(&self.dst)?);
        let output = local_state::DirectoryState::new(dst.clone())
            .with_context(|| format!("failed to create DirectoryState from {dst:?}"))?;
        let root = output.path().to_owned();
        let link = output.link().to_owned();
        Ok(Some(Box::new((
            output,
            GitClone {
                remote: git_utils::Remote::new(&self.url),
                root,
                link,
            },
        ))))
    }
}

#[derive(Clone)]
struct GitCloneBuilder;

impl engine::CommandBuilder for GitCloneBuilder {
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
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (url, dst) = args.expect_double_arg(self.name())?;
        Ok(engine::Command::new_statement(GitCloneStatement {
            url: url.expect_raw().context("url")?.to_owned(),
            dst: dst.clone(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(GitCloneBuilder {}));
}
