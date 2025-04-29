/// Platform-dependent `os_str` <-> [u8] conversions.
///
/// Use for local storage such as local registry,
/// and never transmit across OS boundary.
mod sys;

use std::ffi::OsString;

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("odd number of bytes, must be even")]
    OddNumberOfBytes,
}

pub type Result<T> = std::result::Result<T, Error>;

trait Converter {
    fn from_vec(vec: Vec<u8>) -> Result<OsString>;
    fn to_vec(os_str: OsString) -> Vec<u8>;
}

pub fn from_vec(vec: Vec<u8>) -> Result<OsString> {
    sys::SysConverter::from_vec(vec)
}

pub fn to_vec(os_str: OsString) -> Vec<u8> {
    sys::SysConverter::to_vec(os_str)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use super::*;

    #[test_case("")]
    #[test_case("test")]
    fn test_conversions(text: &str) {
        let orig = OsString::from(text);
        assert_eq!(from_vec(to_vec(orig.clone())).unwrap(), orig);
    }
}
