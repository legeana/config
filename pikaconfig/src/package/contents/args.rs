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
            args![
                "test 1",
                "test 2",
            ],
            Arguments(vec![
                "test 1".to_owned(),
                "test 2".to_owned(),
            ])
        );
    }
}
