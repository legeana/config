use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::process::Command as StdCommand;

#[derive(Debug)]
pub(crate) struct EnvOverlay {
    clear: bool,
    // None means clear, Some(value) means override.
    overrides: HashMap<OsString, Option<OsString>>,
}

impl Default for EnvOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvOverlay {
    pub(crate) fn new() -> Self {
        Self {
            clear: false,
            overrides: HashMap::new(),
        }
    }
    pub(crate) fn clear(&mut self) {
        self.clear = true;
        self.overrides.clear();
    }
    pub(crate) fn insert(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
        self.overrides.insert(
            key.as_ref().to_os_string(),
            Some(value.as_ref().to_os_string()),
        );
    }
    pub(crate) fn remove(&mut self, key: impl AsRef<OsStr>) {
        self.overrides.insert(key.as_ref().to_os_string(), None);
    }
    pub(crate) fn apply(&self, cmd: &mut StdCommand) {
        if self.clear {
            cmd.env_clear();
        }
        for (k, v) in &self.overrides {
            match v {
                Some(value) => cmd.env(k, value),
                None => cmd.env_remove(k),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_clear() {
        let mut env = EnvOverlay::new();
        let mut cmd = StdCommand::new("");
        cmd.env("test", "test-value");

        env.clear();
        env.apply(&mut cmd);

        assert_eq!(cmd.get_envs().collect::<Vec<_>>(), &[]);
    }

    #[test]
    fn test_clear_and_insert() {
        let mut env = EnvOverlay::new();
        let mut cmd = StdCommand::new("");
        cmd.env("test", "test-value");

        env.clear();
        env.insert("foo", "bar");
        env.apply(&mut cmd);

        assert_eq!(
            cmd.get_envs().collect::<Vec<_>>(),
            &[(OsStr::new("foo"), Some(OsStr::new("bar")))],
        );
    }

    #[test]
    fn test_clear_and_remove() {
        let mut env = EnvOverlay::new();
        let mut cmd = StdCommand::new("");
        cmd.env("test-1", "value-1");
        cmd.env("test-2", "value-2");

        env.clear();
        env.remove("test-2");
        env.apply(&mut cmd);

        assert_eq!(cmd.get_envs().collect::<Vec<_>>(), &[]);
    }

    #[test]
    fn test_insert() {
        let mut env = EnvOverlay::new();
        let mut cmd = StdCommand::new("");
        cmd.env("test", "test-value");

        env.insert("foo", "bar");
        env.apply(&mut cmd);

        assert_eq!(
            cmd.get_envs().collect::<Vec<_>>(),
            &[
                (OsStr::new("foo"), Some(OsStr::new("bar"))),
                (OsStr::new("test"), Some(OsStr::new("test-value"))),
            ],
        );
    }

    #[test]
    fn test_insert_and_clear() {
        let mut env = EnvOverlay::new();
        let mut cmd = StdCommand::new("");
        cmd.env("test", "test-value");

        env.insert("foo", "bar");
        env.clear();
        env.apply(&mut cmd);

        assert_eq!(cmd.get_envs().collect::<Vec<_>>(), &[]);
    }

    #[test]
    fn test_remove() {
        let mut env = EnvOverlay::new();
        let mut cmd = StdCommand::new("");
        cmd.env("test-1", "value-1");
        cmd.env("test-2", "value-2");

        env.remove("test-1");
        env.apply(&mut cmd);

        assert_eq!(
            cmd.get_envs().collect::<Vec<_>>(),
            &[
                (OsStr::new("test-1"), None),
                (OsStr::new("test-2"), Some(OsStr::new("value-2"))),
            ],
        );
    }

    #[test]
    fn test_remove_and_clear() {
        let mut env = EnvOverlay::new();
        let mut cmd = StdCommand::new("");
        cmd.env("test-1", "value-1");
        cmd.env("test-2", "value-2");

        env.remove("test-1");
        env.clear();
        env.apply(&mut cmd);

        assert_eq!(cmd.get_envs().collect::<Vec<_>>(), &[]);
    }
}
