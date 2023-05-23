use anyhow::{Context, Result};

use super::local_state;
use super::parser;
use super::util;
use crate::git_utils;
use crate::registry::Registry;

pub struct GitCloneParser {}

const COMMAND: &str = "git_clone";
const BRANCH_SEP: char = '#';

struct Remote {
    url: String,
    #[allow(dead_code)]
    branch: Option<String>,
}

impl Remote {
    fn new(addr: &str) -> Self {
        match addr.rsplit_once(BRANCH_SEP) {
            Some((url, branch)) => Self {
                url: url.to_owned(),
                branch: Some(branch.to_owned()),
            },
            None => Self {
                url: addr.to_owned(),
                branch: None,
            },
        }
    }
}

struct GitClone {
    remote: Remote,
    output: local_state::DirectoryState,
}

impl GitClone {
    fn hard_pull(&self) -> Result<()> {
        git_utils::git_hard_pull(self.output.path())
    }
    fn clone(&self) -> Result<()> {
        git_utils::git_clone(&self.remote.url, self.output.path())
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
        "git_clone <url>[#<branch>] <directory>
           git clone <url> into a local storage and installs a symlink to it
           if <branch> is specified clone <branch> instead of default HEAD"
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
            remote: Remote::new(url),
            output,
        }));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_without_branch() {
        let remote = Remote::new("http://github.com/example/repo.git");
        assert_eq!(remote.url, "http://github.com/example/repo.git");
        assert_eq!(remote.branch, None);
    }

    #[test]
    fn test_remote_with_branch() {
        let remote = Remote::new("http://github.com/example/repo.git#branch");
        assert_eq!(remote.url, "http://github.com/example/repo.git");
        assert_eq!(remote.branch, Some("branch".to_owned()));
    }

    #[test]
    fn test_remote_with_branch_and_hashes() {
        let remote = Remote::new("http://github.com/#example/repo.git#branch");
        assert_eq!(remote.url, "http://github.com/#example/repo.git");
        assert_eq!(remote.branch, Some("branch".to_owned()));
    }
}
