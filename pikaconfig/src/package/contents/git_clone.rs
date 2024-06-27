use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::annotated_path::AnnotatedPathBox;
use crate::git_utils;
use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

struct GitClone {
    remote: git_utils::Remote,
    repo: AnnotatedPathBox,
}

impl GitClone {
    fn need_update(&self) -> Result<bool> {
        let remote_url = git_utils::get_remote_url(self.repo.as_path())?;
        if remote_url != self.remote.url {
            return Ok(true);
        }
        let target_branch = match self.remote.branch {
            Some(ref branch) => branch.clone(),
            None => git_utils::get_remote_head_ref(self.repo.as_path())?,
        };
        let current_branch = git_utils::get_head_ref(self.repo.as_path())?;
        Ok(target_branch != current_branch)
    }
    fn force_pull(&self) -> Result<()> {
        git_utils::git_force_shallow_pull(self.repo.as_path(), &self.remote)
    }
    fn clone(&self) -> Result<()> {
        git_utils::git_shallow_clone(&self.remote, self.repo.as_path())
    }
    fn is_empty(&self) -> Result<bool> {
        let count = std::fs::read_dir(self.repo.as_path())
            .with_context(|| format!("failed to read dir {:?}", self.repo))?
            .count();
        Ok(count == 0)
    }
}

impl Module for GitClone {
    fn install(&self, rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        if self.is_empty()? {
            self.clone()
                .with_context(|| format!("failed to git clone into {:?}", self.repo))
        } else {
            if !self.need_update()? && !rules.force_update {
                return Ok(());
            }
            self.force_pull()
                .with_context(|| format!("failed to git pull {:?}", self.repo))
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
        let output = local_state::dir_state(dst.clone())
            .with_context(|| format!("failed to create DirectoryState from {dst:?}"))?;
        let repo = output.state();
        Ok(Some(Box::new((
            output,
            GitClone {
                remote: git_utils::Remote::new(&self.url),
                repo,
            },
        ))))
    }
}

#[derive(Debug)]
struct GitExpression {
    workdir: PathBuf,
    url: String,
}

impl engine::Expression for GitExpression {
    fn eval(&self, _ctx: &mut engine::Context) -> Result<engine::ExpressionOutput> {
        let output = local_state::dir_cache(&self.workdir, Path::new(""), &self.url)?;
        let output_state = output.state();
        let git_clone = GitClone {
            remote: git_utils::Remote::new(&self.url),
            repo: output.state(),
        };
        Ok(engine::ExpressionOutput {
            module: Some(Box::new((output, git_clone))),
            output: output_state.as_path().to_owned().into_os_string(),
        })
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

#[derive(Clone)]
struct GitBuilder;

impl engine::CommandBuilder for GitBuilder {
    fn name(&self) -> String {
        "git".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            <directory> = {command} <url>[#<branch>]
                Clones a remote git repository,
                returns the path to the local clone.
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let url = args
            .expect_single_arg(self.name())?
            .expect_raw()
            .context("url")?
            .to_owned();
        Ok(engine::Command::new_expression(GitExpression {
            workdir: workdir.to_owned(),
            url,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(GitCloneBuilder));
    registry.register_command(Box::new(GitBuilder));
}
