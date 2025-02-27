/// Platform-dependent os_str <-> [u8] conversions.
///
/// Use for local storage such as local registry,
/// and never transmit across OS boundary.
use std::ffi::OsString;

use anyhow::Result;

#[cfg(any(test, windows))]
mod windows {
    use thiserror::Error;

    #[derive(Debug, Error, PartialEq)]
    pub(super) enum Error {
        #[error("odd number of bytes, must be even")]
        OddNumberOfBytes,
    }

    pub(super) type Result<T> = std::result::Result<T, Error>;

    pub(super) fn to_wide(b: [u8; 2]) -> u16 {
        u16::from_le_bytes(b)
    }

    pub(super) fn from_wide(b: u16) -> [u8; 2] {
        b.to_le_bytes()
    }

    pub(super) fn to_wide_vec(b: Vec<u8>) -> Result<Vec<u16>> {
        if b.len() % 2 != 0 {
            return Err(Error::OddNumberOfBytes);
        }
        let mut r = Vec::<u16>::with_capacity(b.len() / 2);
        for c in b.chunks(2) {
            if c.len() != 2 {
                return Err(Error::OddNumberOfBytes);
            }
            r.push(to_wide([c[0], c[1]]));
        }
        Ok(r)
    }

    pub(super) fn from_wide_iter(w: impl Iterator<Item = u16>) -> Vec<u8> {
        w.flat_map(from_wide).collect()
    }
}

#[cfg(unix)]
mod platform {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt as _;

    use anyhow::Result;

    pub(super) fn from_vec(vec: Vec<u8>) -> Result<OsString> {
        Ok(OsString::from_vec(vec))
    }

    pub(super) fn to_vec(os_str: OsString) -> Vec<u8> {
        os_str.into_vec()
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStrExt as _;
    use std::os::windows::ffi::OsStringExt as _;

    use anyhow::Result;

    use super::windows;

    pub(super) fn from_vec(vec: Vec<u8>) -> Result<OsString> {
        let w = windows::to_wide_vec(vec)?;
        Ok(OsString::from_wide(&w))
    }

    pub(super) fn to_vec(os_str: OsString) -> Vec<u8> {
        windows::from_wide_iter(os_str.encode_wide())
    }
}

pub fn from_vec(vec: Vec<u8>) -> Result<OsString> {
    platform::from_vec(vec)
}

pub fn to_vec(os_str: OsString) -> Vec<u8> {
    platform::to_vec(os_str)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_to_wide() {
        assert_eq!(windows::to_wide([0x00, 0x10]), 0x1000);
        assert_eq!(windows::to_wide([0x10, 0x00]), 0x0010);
        assert_eq!(windows::to_wide([0x01, 0x02]), 0x0201);
    }

    #[test]
    fn test_from_wide() {
        assert_eq!(windows::from_wide(0x1000), [0x00, 0x10]);
        assert_eq!(windows::from_wide(0x0010), [0x10, 0x00]);
        assert_eq!(windows::from_wide(0x1020), [0x20, 0x10]);
    }

    #[test]
    fn test_to_wide_vec() {
        assert_eq!(
            windows::to_wide_vec(vec![0x00, 0x01, 0x02, 0x03]).unwrap(),
            vec![0x0100, 0x0302],
        );
    }

    #[test]
    fn test_to_wide_vec_odd() {
        assert_eq!(
            windows::to_wide_vec(vec![0x00]),
            Err(windows::Error::OddNumberOfBytes),
        );
    }

    #[test]
    fn test_from_wide_iter() {
        assert_eq!(
            windows::from_wide_iter(vec![0x0100, 0x0302].into_iter()),
            vec![0x00, 0x01, 0x02, 0x03],
        );
    }

    #[test]
    fn test_conversions() {
        let orig = OsString::from("test");
        assert_eq!(from_vec(to_vec(orig.clone())).unwrap(), orig);
    }
}
