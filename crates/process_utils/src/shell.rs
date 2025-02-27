use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use anyhow::Result;

use crate::command::Command;
use crate::env::EnvOverlay;
use crate::process_utils;

#[derive(Debug)]
pub struct Shell {
    current_dir: Option<PathBuf>,
    env: EnvOverlay,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Self {
        Self {
            current_dir: None,
            env: EnvOverlay::new(),
        }
    }
    // Configuration.
    pub fn current_dir(&mut self, dir: impl AsRef<Path>) {
        self.current_dir = Some(dir.as_ref().to_path_buf());
    }
    pub fn clear_current_dir(&mut self) {
        self.current_dir = None;
    }
    pub fn env(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
        self.env.insert(key, value);
    }
    pub fn envs<I, K, V>(&mut self, vars: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (key, value) in vars {
            self.env.insert(key, value);
        }
    }
    pub fn env_remove(&mut self, key: impl AsRef<OsStr>) {
        self.env.remove(key);
    }
    pub fn env_clear(&mut self) {
        self.env.clear();
    }
    fn finalise(&self, cmd: Command) -> StdCommand {
        cmd.finalise_with(self.current_dir.as_ref(), &self.env)
    }
    pub fn run(&self, cmd: Command) -> Result<()> {
        process_utils::run(&mut self.finalise(cmd))
    }
    pub fn run_verbose(&self, cmd: Command) -> Result<()> {
        process_utils::run_verbose(&mut self.finalise(cmd))
    }
    pub fn output(&self, cmd: Command) -> Result<String> {
        process_utils::output(&mut self.finalise(cmd))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_env() {
        let mut sh = Shell::new();

        sh.env("env-1", "value-1");

        let std_cmd = sh.finalise(Command::new(""));
        assert_eq!(
            std_cmd.get_envs().collect::<Vec<_>>(),
            &[(OsStr::new("env-1"), Some(OsStr::new("value-1")))],
        );
    }

    #[test]
    fn test_envs() {
        let mut sh = Shell::new();

        sh.envs([("env-1", "value-1"), ("env-2", "value-2")]);

        let std_cmd = sh.finalise(Command::new(""));
        assert_eq!(
            std_cmd.get_envs().collect::<Vec<_>>(),
            &[
                (OsStr::new("env-1"), Some(OsStr::new("value-1"))),
                (OsStr::new("env-2"), Some(OsStr::new("value-2"))),
            ],
        );
    }

    #[test]
    fn test_env_remove() {
        let mut sh = Shell::new();

        sh.env_remove("env-1");

        let std_cmd = sh.finalise(Command::new(""));
        assert_eq!(
            std_cmd.get_envs().collect::<Vec<_>>(),
            &[(OsStr::new("env-1"), None)],
        );
    }

    #[test]
    fn test_env_clear() {
        let mut sh = Shell::new();
        sh.env("env-1", "value-1");

        sh.env_clear();

        let std_cmd = sh.finalise(Command::new(""));
        assert_eq!(std_cmd.get_envs().collect::<Vec<_>>(), &[]);
    }
}
