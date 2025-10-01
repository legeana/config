use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt as _;
use std::os::windows::ffi::OsStringExt as _;

use crate::os_str::{Converter, Error, Result};

fn to_wide(b: [u8; 2]) -> u16 {
    u16::from_le_bytes(b)
}

fn from_wide(b: u16) -> [u8; 2] {
    b.to_le_bytes()
}

fn to_wide_vec(b: &[u8]) -> Result<Vec<u16>> {
    if !b.len().is_multiple_of(2) {
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

fn from_wide_iter(w: impl Iterator<Item = u16>) -> Vec<u8> {
    w.flat_map(from_wide).collect()
}

pub(in crate::os_str) struct SysConverter;

impl Converter for SysConverter {
    fn from_vec(vec: Vec<u8>) -> Result<OsString> {
        let w = to_wide_vec(&vec)?;
        Ok(OsString::from_wide(&w))
    }
    fn to_vec(os_str: OsString) -> Vec<u8> {
        from_wide_iter(os_str.encode_wide())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use super::*;

    #[test_case("", &[])]
    #[test_case(
        "hello",
        // UTF-16
        &[
            b'h', 0x00,
            b'e', 0x00,
            b'l', 0x00,
            b'l', 0x00,
            b'o', 0x00,
        ])]
    fn test_sanity(text: &str, bytes: &[u8]) {
        let os_text = OsString::from(text);
        assert_eq!(SysConverter::from_vec(bytes.to_vec()), Ok(os_text.clone()));
        assert_eq!(SysConverter::to_vec(os_text), bytes);
    }

    #[test]
    fn test_to_wide() {
        assert_eq!(to_wide([0x00, 0x10]), 0x1000);
        assert_eq!(to_wide([0x10, 0x00]), 0x0010);
        assert_eq!(to_wide([0x01, 0x02]), 0x0201);
    }

    #[test]
    fn test_from_wide() {
        assert_eq!(from_wide(0x1000), [0x00, 0x10]);
        assert_eq!(from_wide(0x0010), [0x10, 0x00]);
        assert_eq!(from_wide(0x1020), [0x20, 0x10]);
    }

    #[test]
    fn test_to_wide_vec() {
        assert_eq!(
            to_wide_vec(&[0x00, 0x01, 0x02, 0x03]).unwrap(),
            vec![0x0100, 0x0302],
        );
    }

    #[test]
    fn test_to_wide_vec_odd() {
        assert_eq!(to_wide_vec(&[0x00]), Err(Error::OddNumberOfBytes));
    }

    #[test]
    fn test_from_wide_iter() {
        assert_eq!(
            from_wide_iter([0x0100, 0x0302].into_iter()),
            vec![0x00, 0x01, 0x02, 0x03],
        );
    }
}
