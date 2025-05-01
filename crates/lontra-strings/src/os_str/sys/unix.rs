use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt as _;

use crate::os_str::{Converter, Result};

pub(in crate::os_str) struct SysConverter;

impl Converter for SysConverter {
    fn from_vec(vec: Vec<u8>) -> Result<OsString> {
        Ok(OsString::from_vec(vec))
    }
    fn to_vec(os_str: OsString) -> Vec<u8> {
        os_str.into_vec()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use super::*;

    #[test_case("", b"")]
    #[test_case("hello", b"hello")]
    fn test_sanity(text: &str, bytes: &[u8]) {
        let os_text = OsString::from(text);
        assert_eq!(SysConverter::from_vec(bytes.to_vec()), Ok(os_text.clone()));
        assert_eq!(SysConverter::to_vec(os_text), bytes);
    }
}
