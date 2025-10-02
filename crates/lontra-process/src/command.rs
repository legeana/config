use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use anyhow::{Context as _, Result};

use crate::env::EnvOverlay;
use crate::{Shell, process_utils};

#[derive(Debug)]
pub struct Command {
    inner: StdCommand,
    current_dir: Option<PathBuf>,
    env: EnvOverlay,
}

impl Command {
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Self {
            inner: StdCommand::new(program),
            current_dir: None,
            env: EnvOverlay::new(),
        }
    }
    pub fn from_argv<I, S>(argv: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut it = argv.into_iter();
        let program = it.next().context("empty argv")?;
        let c = Self::new(program);
        Ok(c.args(it))
    }
    #[must_use]
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.inner.arg(arg);
        self
    }
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }
    #[must_use]
    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.current_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    #[must_use]
    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.env.insert(key, value);
        self
    }
    #[must_use]
    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (key, value) in vars {
            self.env.insert(key, value);
        }
        self
    }
    #[must_use]
    pub fn env_remove(mut self, key: impl AsRef<OsStr>) -> Self {
        self.env.remove(key);
        self
    }
    #[must_use]
    pub fn env_clear(mut self) -> Self {
        self.env.clear();
        self
    }
    pub(crate) fn finalise(mut self) -> StdCommand {
        if let Some(dir) = self.current_dir {
            self.inner.current_dir(dir);
        }
        self.env.apply(&mut self.inner);
        self.inner
    }
    pub(crate) fn finalise_with<P>(
        mut self,
        base_dir: Option<P>,
        base_env: &EnvOverlay,
    ) -> StdCommand
    where
        P: AsRef<Path>,
    {
        if let Some(dir) = base_dir {
            self.inner.current_dir(dir);
        }
        base_env.apply(&mut self.inner);
        self.finalise()
    }
    // Direct run helpers.
    pub fn run(self) -> Result<()> {
        process_utils::run(&mut self.finalise())
    }
    pub fn run_verbose(self) -> Result<()> {
        process_utils::run_verbose(&mut self.finalise())
    }
    pub fn output(self) -> Result<String> {
        process_utils::output(&mut self.finalise())
    }
    // Run helpers chained with Shell.
    pub fn run_in(self, sh: &Shell) -> Result<()> {
        sh.run(self)
    }
    pub fn run_verbose_in(self, sh: &Shell) -> Result<()> {
        sh.run_verbose(self)
    }
    pub fn output_in(self, sh: &Shell) -> Result<String> {
        sh.output(self)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use super::*;

    type FinaliseFn = fn(Command) -> StdCommand;

    fn finalise(cmd: Command) -> StdCommand {
        cmd.finalise()
    }

    fn finalise_with_default(cmd: Command) -> StdCommand {
        cmd.finalise_with(None::<&Path>, &EnvOverlay::new())
    }

    #[test_case(finalise)]
    #[test_case(finalise_with_default)]
    fn test_finalise_program(finalise: FinaliseFn) {
        let cmd = Command::new("test");

        let std_cmd = finalise(cmd);

        assert_eq!(std_cmd.get_program(), "test");
    }

    #[test]
    fn test_finalise_with_program() {
        let cmd = Command::new("test");

        let std_cmd = cmd.finalise_with(None::<&Path>, &EnvOverlay::new());

        assert_eq!(std_cmd.get_program(), "test");
    }

    #[test_case(None, None)]
    #[test_case(Some("/root"), Some("/root"))]
    fn test_finalise_current_dir(current_dir: Option<&str>, want: Option<&str>) {
        let mut cmd = Command::new("");
        if let Some(dir) = current_dir {
            cmd = cmd.current_dir(dir);
        }

        let std_cmd = cmd.finalise();

        assert_eq!(std_cmd.get_current_dir(), want.map(Path::new));
    }

    #[test_case(None, None, None)]
    #[test_case(Some("/root"), None, Some("/root"))]
    #[test_case(None, Some("/root"), Some("/root"))]
    #[test_case(Some("/usr"), Some("/root"), Some("/root"))]
    fn test_finalise_with_current_dir(
        base: Option<&str>,
        current_dir: Option<&str>,
        want: Option<&str>,
    ) {
        let mut cmd = Command::new("");
        if let Some(dir) = current_dir {
            cmd = cmd.current_dir(dir);
        }

        let std_cmd = cmd.finalise_with(base, &EnvOverlay::new());

        assert_eq!(std_cmd.get_current_dir(), want.map(Path::new));
    }

    #[test_case(finalise)]
    #[test_case(finalise_with_default)]
    fn test_finalise_args(finalise: FinaliseFn) {
        let cmd = Command::new("").arg("arg-1").args(["arg-2", "arg-3"]);

        let std_cmd = finalise(cmd);

        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_from_argv_empty() {
        let empty: &[&str] = &[];
        let cmd = Command::from_argv(empty);

        assert!(matches!(cmd, Result::Err(_)));
    }

    #[test_case(&["program"], "program", &[])]
    #[test_case(&["program", "arg 1"], "program", &["arg 1"])]
    #[test_case(&["program", "arg 1", "arg 2"], "program", &["arg 1", "arg 2"])]
    fn test_from_argv(argv: &[&str], want_program: &str, want_args: &[&str]) {
        let cmd = Command::from_argv(argv).expect("from_argv");

        let std_cmd = finalise(cmd);

        assert_eq!(std_cmd.get_program(), want_program);
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), want_args);
    }

    #[test]
    fn test_finalise_with_env_combines() {
        let cmd = Command::new("").env("env-1", "value-1");
        let mut base_env = EnvOverlay::new();
        base_env.insert("env-2", "base-2");

        let std_cmd = cmd.finalise_with(None::<&Path>, &base_env);

        assert_eq!(
            std_cmd.get_envs().collect::<Vec<_>>(),
            [
                (OsStr::new("env-1"), Some(OsStr::new("value-1"))),
                (OsStr::new("env-2"), Some(OsStr::new("base-2"))),
            ],
        );
    }

    #[test]
    fn test_finalise_with_env_overrides() {
        let cmd = Command::new("").env("env-1", "value-1");
        let mut base_env = EnvOverlay::new();
        base_env.insert("env-1", "base-1");

        let std_cmd = cmd.finalise_with(None::<&Path>, &base_env);

        assert_eq!(
            std_cmd.get_envs().collect::<Vec<_>>(),
            [(OsStr::new("env-1"), Some(OsStr::new("value-1"))),],
        );
    }

    #[test_case(finalise)]
    #[test_case(finalise_with_default)]
    fn test_finalise_envs(finalise: FinaliseFn) {
        let cmd = Command::new("").envs([("env-1", "value-1"), ("env-2", "value-2")]);

        let std_cmd = finalise(cmd);

        assert_eq!(
            std_cmd.get_envs().collect::<Vec<_>>(),
            [
                (OsStr::new("env-1"), Some(OsStr::new("value-1"))),
                (OsStr::new("env-2"), Some(OsStr::new("value-2"))),
            ],
        );
    }
}
