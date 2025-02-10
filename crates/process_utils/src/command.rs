use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use anyhow::Result;

use crate::env::EnvOverlay;
use crate::process_utils;

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
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.inner.arg(arg);
        self
    }
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }
    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.current_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.env.insert(key, value);
        self
    }
    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (key, value) in vars.into_iter() {
            self.env.insert(key, value);
        }
        self
    }
    pub fn env_remove(mut self, key: impl AsRef<OsStr>) -> Self {
        self.env.remove(key);
        self
    }
    pub fn env_clear(mut self) -> Self {
        self.env.clear();
        self
    }
    fn finalise(mut self) -> StdCommand {
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
    pub fn run(self) -> Result<()> {
        process_utils::run(&mut self.finalise())
    }
    pub fn run_verbose(self) -> Result<()> {
        process_utils::run_verbose(&mut self.finalise())
    }
    pub fn output(self) -> Result<String> {
        process_utils::output(&mut self.finalise())
    }
}

#[macro_export]
macro_rules! cmd {
    [@new $program:expr] => { $crate::Command::new($program) };

    [@push_down () -> ($($body:tt)*)] => { $($body)* };
    [@push_down ([] $(,)*) -> ($($body:tt)*)] => { $($body)* };
    [@push_down ([$($arg:expr),* $(,)*] $(,)*) -> ($($body:tt)*)] => {
        $($body)* $(.arg($arg))*
    };
    [@push_down ([], $($tail:tt)*) -> $body:tt] => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            $body
        ]
    };
    [@push_down ([$($arg:expr),* $(,)*], $($tail:tt)*) -> ($($body:tt)*)] => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            ($($body)* $(.arg($arg))*)
        ]
    };
    [@push_down ($arg:expr) -> ($($body:tt)*)] => {
        $($body)*.args($arg)
    };
    [@push_down ($arg:expr, $($tail:tt)*) -> ($($body:tt)*)] => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            ($($body)*.args($arg))
        ]
    };

    ($program:expr) => { cmd![@new $program] };
    ($program:expr, $($tail:tt)*) => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            (cmd![@new $program])
        ]
    };
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
        let cmd = Command::new("").arg("arg-1").args(&["arg-2", "arg-3"]);

        let std_cmd = finalise(cmd);

        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
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

    #[test]
    fn test_cmd_empty() {
        let cmd = cmd!("program");

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &[] as &[&str]);
    }

    #[test]
    fn test_cmd_single_arg() {
        let cmd = cmd!("program", ["arg-1"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &["arg-1"]);
    }

    #[test]
    fn test_cmd_single_arg_trailing_comma() {
        let cmd = cmd!("program", ["arg-1"],);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &["arg-1"]);
    }

    #[test]
    fn test_cmd_multiple_args() {
        let cmd = cmd!("program", ["arg-1", "arg-2", "arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_different_arg_types() {
        let arg1 = "arg-1";
        let arg2 = Path::new("arg-2");
        let arg3 = OsStr::new("arg-3");

        let cmd = cmd!("program", [arg1, arg2, arg3]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_multiple_args_trailing_comma() {
        let cmd = cmd!("program", ["arg-1", "arg-2", "arg-3"],);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_args() {
        let cmd = cmd!("program", vec!["arg-1", "arg-2"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &["arg-1", "arg-2"]);
    }

    #[test]
    fn test_cmd_args_trailing_comma() {
        let cmd = cmd!("program", vec!["arg-1", "arg-2"],);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &["arg-1", "arg-2"]);
    }

    #[test]
    fn test_cmd_args_before_arg() {
        let cmd = cmd!("program", vec!["arg-1", "arg-2"], ["arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_args_after_arg() {
        let cmd = cmd!("program", ["arg-1"], vec!["arg-2", "arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }
}
