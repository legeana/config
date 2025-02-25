use std::borrow::Cow;

use anyhow::{Result, anyhow};

#[macro_export]
macro_rules! args {
    (@as_raw $e:expr) => {
        $crate::package::contents::args::Argument::Raw($e.into())
    };
    (@as_vars $e:expr) => {
        $crate::package::contents::args::Argument::OnlyVars($e.into())
    };
    (@as_vars_and_home $e:expr) => {
        $crate::package::contents::args::Argument::VarsAndHome($e.into())
    };
    [@build $($body:tt)*] => {
        $crate::package::contents::args::Arguments(vec![$($body)*])
    };
    [@push_down () -> ($($body:tt)*)] => {
        args![@build $($body)*]
    };
    [@push_down (@ $e:expr $(, $($tail:tt)*)?) -> ($($body:tt)*)] => {
        args!(
            @push_down
            ($($($tail)*)?)
            ->
            ($($body)* args!(@as_vars $e),)
        )
    };
    [@push_down (~ $e:expr $(, $($tail:tt)*)?) -> ($($body:tt)*)] => {
        args!(
            @push_down
            ($($($tail)*)?)
            ->
            ($($body)* args!(@as_vars_and_home $e),)
        )
    };
    [@push_down ($e:expr $(, $($tail:tt)*)?) -> ($($body:tt)*)] => {
        args!(
            @push_down
            ($($($tail)*)?)
            ->
            ($($body)* args!(@as_raw $e),)
        )
    };
    [$($body:tt)*] => {
        args!(
            @push_down
            ($($body)*)
            ->
            ()
        )
    };
}

#[allow(unused_imports)]
pub(super) use args;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Argument {
    Raw(String),
    OnlyVars(String),
    VarsAndHome(String),
}

impl Argument {
    pub(super) fn expect_raw(&self) -> Result<&str> {
        let get_env = |_: &_| -> Result<Option<String>, anyhow::Error> {
            Err(anyhow!("can't use string template in this context"))
        };
        let result = match self {
            Self::Raw(s) => return Ok(s),
            Self::OnlyVars(t) => shellexpand::env_with_context(t, get_env),
            Self::VarsAndHome(t) => {
                let mut called_home = false;
                let home_dir = || -> Option<String> {
                    called_home = true;
                    None
                };
                let raw_result = shellexpand::full_with_context(t, home_dir, get_env);
                if called_home {
                    return Err(anyhow!(
                        "failed to coerce template to a raw string: ~ expansion is not allowed"
                    ));
                }
                raw_result
            }
        };
        match result {
            Err(e) => Err(anyhow!("failed to coerce template to a raw string: {e}")),
            Ok(Cow::Borrowed(s)) => Ok(s),
            Ok(Cow::Owned(_)) => Err(anyhow!("can't use string template in this context")),
        }
    }
}

impl std::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Raw(s) => s,
            Self::OnlyVars(s) => s,
            Self::VarsAndHome(s) => s,
        };
        write!(f, "{}", shlexfmt::quote(s))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Arguments(pub Vec<Argument>);

impl Arguments {
    pub(super) fn expect_no_args(&self, command: impl AsRef<str>) -> Result<()> {
        let command = command.as_ref();
        if !self.0.is_empty() {
            return Err(anyhow!(
                "{} builder: want no arguments, got {}: {:?}",
                command,
                self.0.len(),
                self.0,
            ));
        }
        Ok(())
    }

    pub(super) fn expect_any_args(&self, command: impl AsRef<str>) -> Result<&[Argument]> {
        let (_, args) = self.expect_variadic_args(command, 0)?;
        Ok(args)
    }

    pub(super) fn expect_optional_arg(
        &self,
        command: impl AsRef<str>,
    ) -> Result<Option<&Argument>> {
        let command = command.as_ref();
        let (_, args) = self.expect_variadic_args(command, 0)?;
        match args.len() {
            0 => Ok(None),
            1 => Ok(Some(&args[0])),
            len => Err(anyhow!(
                "{command} builder: want an optional argument, got {}: {:?}",
                len,
                args,
            )),
        }
    }

    pub(super) fn expect_single_arg(&self, command: impl AsRef<str>) -> Result<&Argument> {
        Ok(&self.expect_fixed_args(command, 1)?[0])
    }

    pub(super) fn expect_at_least_one_arg(
        &self,
        command: impl AsRef<str>,
    ) -> Result<(&Argument, &[Argument])> {
        let (arg, tail) = self.expect_variadic_args(command, 1)?;
        assert_eq!(arg.len(), 1);
        Ok((&arg[0], tail))
    }

    pub(super) fn expect_double_arg(
        &self,
        command: impl AsRef<str>,
    ) -> Result<(&Argument, &Argument)> {
        let args = self.expect_fixed_args(command, 2)?;
        Ok((&args[0], &args[1]))
    }

    fn expect_fixed_args(&self, command: impl AsRef<str>, len: usize) -> Result<&[Argument]> {
        let command = command.as_ref();
        if self.0.len() != len {
            return Err(anyhow!(
                "{command} builder: want {len} arguments, got {}: {:?}",
                self.0.len(),
                self.0,
            ));
        }
        Ok(&self.0)
    }

    /// Returns (required_args, remainder_args).
    fn expect_variadic_args(
        &self,
        command: impl AsRef<str>,
        required: usize,
    ) -> Result<(&[Argument], &[Argument])> {
        let command = command.as_ref();
        if self.0.len() < required {
            return Err(anyhow!(
                "{} builder: want at least {} arguments, got {}: {:?}",
                command,
                required,
                self.0.len(),
                self.0,
            ));
        }
        let required_args = &self.0[0..required];
        let remainder_args = &self.0[required..];
        Ok((required_args, remainder_args))
    }
}

impl AsRef<[Argument]> for Arguments {
    fn as_ref(&self) -> &[Argument] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expect_raw_from_raw() {
        assert_eq!(Argument::Raw("test".into()).expect_raw().unwrap(), "test");
    }

    #[test]
    fn test_expect_raw_from_template() {
        // "test"
        assert_eq!(
            Argument::OnlyVars("test".into()).expect_raw().unwrap(),
            "test"
        );
        assert_eq!(
            Argument::VarsAndHome("test".into()).expect_raw().unwrap(),
            "test"
        );
        // "test~"
        assert_eq!(
            Argument::OnlyVars("test~".into()).expect_raw().unwrap(),
            "test~"
        );
        assert_eq!(
            Argument::VarsAndHome("test~".into()).expect_raw().unwrap(),
            "test~"
        );
        // "~/test"
        assert_eq!(
            Argument::OnlyVars("~/test".into()).expect_raw().unwrap(),
            "~/test"
        );
        assert!(Argument::VarsAndHome("~/test".into()).expect_raw().is_err());
        // "${test}"
        assert!(Argument::OnlyVars("${test}".into()).expect_raw().is_err());
        assert!(
            Argument::VarsAndHome("${test}".into())
                .expect_raw()
                .is_err()
        );
    }

    #[test]
    fn test_empty_args() {
        assert_eq!(args![], Arguments(vec![]));
    }

    #[test]
    fn test_single_arg() {
        assert_eq!(
            args!["test"],
            Arguments(vec![Argument::Raw("test".to_owned())])
        );
        assert_eq!(
            args![@"test"],
            Arguments(vec![Argument::OnlyVars("test".to_owned())])
        );
        assert_eq!(
            args![~"test"],
            Arguments(vec![Argument::VarsAndHome("test".to_owned())])
        );
    }

    #[test]
    fn test_single_arg_trailing_comma() {
        assert_eq!(
            args!["test",],
            Arguments(vec![Argument::Raw("test".to_owned())])
        );
        assert_eq!(
            args![@"test",],
            Arguments(vec![Argument::OnlyVars("test".to_owned())])
        );
        assert_eq!(
            args![~"test",],
            Arguments(vec![Argument::VarsAndHome("test".to_owned())])
        );
    }

    #[test]
    fn test_multiple_args() {
        assert_eq!(
            args!["test 1", @"test 2", ~"test 3"],
            Arguments(vec![
                Argument::Raw("test 1".to_owned()),
                Argument::OnlyVars("test 2".to_owned()),
                Argument::VarsAndHome("test 3".to_owned()),
            ])
        );
    }

    #[test]
    fn test_multiple_args_trailing_comma() {
        assert_eq!(
            args!["test 1", "test 2",],
            Arguments(vec![
                Argument::Raw("test 1".to_owned()),
                Argument::Raw("test 2".to_owned()),
            ])
        );
    }
}
