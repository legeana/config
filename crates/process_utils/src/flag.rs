use std::ffi::{OsStr, OsString};

pub fn opt_flag<V>(name: impl AsRef<OsStr>, value: Option<V>) -> Vec<OsString>
where
    V: AsRef<OsStr>,
{
    match value {
        Some(value) => vec![name.as_ref().to_os_string(), value.as_ref().to_os_string()],
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_some() {
        let args = opt_flag("--test", Some("value"));

        assert_eq!(args, vec!["--test", "value"]);
    }

    #[test]
    fn test_none() {
        let args = opt_flag("--test", None::<&OsStr>);

        assert_eq!(args, Vec::<&OsStr>::new());
    }
}
