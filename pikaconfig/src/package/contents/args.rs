use anyhow::{anyhow, Result};

#[macro_export]
macro_rules! args {
    ($($x:expr,)*) => {
        $crate::package::contents::args::Arguments(vec![$($x.into(),)*])
    };
    ($($x:expr),*) => {
        args![$($x,)*]
    };
}

#[allow(unused_imports)]
pub(crate) use args;

#[derive(Clone, Debug, PartialEq)]
pub struct Arguments(pub Vec<String>);

impl Arguments {
    pub fn expect_no_args(&self, command: impl AsRef<str>) -> Result<()> {
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

    pub fn expect_single_arg(&self, command: impl AsRef<str>) -> Result<&str> {
        Ok(&self.expect_fixed_args(command, 1)?[0])
    }

    pub fn expect_double_arg(&self, command: impl AsRef<str>) -> Result<(&str, &str)> {
        let args = self.expect_fixed_args(command, 2)?;
        Ok((&args[0], &args[1]))
    }

    pub fn expect_fixed_args(&self, command: impl AsRef<str>, len: usize) -> Result<&[String]> {
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
    pub fn expect_variadic_args(
        &self,
        command: impl AsRef<str>,
        required: usize,
    ) -> Result<(&[String], &[String])> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_args() {
        assert_eq!(args![], Arguments(vec![]));
    }

    #[test]
    fn test_single_arg() {
        assert_eq!(args!["test"], Arguments(vec!["test".to_owned()]));
    }

    #[test]
    fn test_single_arg_trailing_comma() {
        assert_eq!(args!["test",], Arguments(vec!["test".to_owned()]));
    }

    #[test]
    fn test_multiple_args() {
        assert_eq!(
            args!["test 1", "test 2"],
            Arguments(vec!["test 1".to_owned(), "test 2".to_owned()])
        );
    }

    #[test]
    fn test_multiple_args_trailing_comma() {
        assert_eq!(
            args!["test 1", "test 2",],
            Arguments(vec!["test 1".to_owned(), "test 2".to_owned(),])
        );
    }
}
