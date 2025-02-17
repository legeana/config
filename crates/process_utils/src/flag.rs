use std::ffi::{OsStr, OsString};

use either::{for_both, Either, Left, Right};

/// Either the name of a flag or its value.
#[derive(Debug, PartialEq)]
pub struct FlagArg<N, V>(Either<N, V>);

impl<N, V> FlagArg<N, V> {
    fn new_name(name: N) -> Self {
        Self(Left(name))
    }
    fn new_value(value: V) -> Self {
        Self(Right(value))
    }
}

/// Arguments are converted into OsStr.
impl<N, V> AsRef<OsStr> for FlagArg<N, V>
where
    N: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    fn as_ref(&self) -> &OsStr {
        for_both!(&self.0, v => v.as_ref())
    }
}

/// Arguments can be converted into OsString.
impl<N, V> From<FlagArg<N, V>> for OsString
where
    N: Into<OsString>,
    V: Into<OsString>,
{
    fn from(value: FlagArg<N, V>) -> Self {
        for_both!(value.0, v => v.into())
    }
}

#[derive(Debug)]
pub struct Flag<I> {
    name: I,
    value: Option<I>,
}

impl<I> IntoIterator for Flag<I> {
    type Item = I;
    type IntoIter = IntoIter<I>;

    fn into_iter(self) -> Self::IntoIter {
        let Some(value) = self.value else {
            return Self::IntoIter::End;
        };
        Self::IntoIter::Name(self.name, value)
    }
}

// Each state is (next, tail*).
pub enum IntoIter<I> {
    Name(I, I),
    Value(I),
    End,
}

impl<I> Iterator for IntoIter<I> {
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        let state = std::mem::replace(self, Self::End);
        match state {
            Self::Name(name, value) => {
                *self = Self::Value(value);
                Some(name)
            }
            Self::Value(value) => {
                *self = Self::End;
                Some(value)
            }
            Self::End => None,
        }
    }
}

#[inline]
pub fn opt_flag<N, V>(name: N, value: Option<V>) -> Flag<FlagArg<N, V>> {
    Flag {
        name: FlagArg::new_name(name),
        value: value.map(FlagArg::new_value),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_some_as_ref() {
        let args = opt_flag("--test", Some("value"));

        let result: Vec<_> = args
            .into_iter()
            .map(|v| v.as_ref().to_os_string())
            .collect();
        assert_eq!(result, vec!["--test", "value"]);
    }

    #[test]
    fn test_none_as_ref() {
        let args = opt_flag("--test", None::<&OsStr>);

        let result: Vec<_> = args
            .into_iter()
            .map(|v| v.as_ref().to_os_string())
            .collect();
        assert_eq!(result, Vec::<&OsStr>::new());
    }

    #[test]
    fn test_some_into() {
        let args = opt_flag("--test", Some("value"));

        let result: Vec<_> = args.into_iter().map(OsString::from).collect();
        assert_eq!(result, vec!["--test", "value"]);
    }

    #[test]
    fn test_none_into() {
        let args = opt_flag("--test", None::<&OsStr>);

        let result: Vec<_> = args.into_iter().map(OsString::from).collect();
        assert_eq!(result, Vec::<&OsStr>::new());
    }
}
