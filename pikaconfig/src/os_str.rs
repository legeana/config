/// Platform-dependent os_str <-> [u8] conversions.
///
/// Use for local storage such as local registry,
/// and never transmit across OS boundary.
use std::ffi::OsString;

use anyhow::Result;

#[cfg(unix)]
mod platform {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

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
    compile_error!("Windows is not yet supported");
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
    fn test_conversions() {
        let orig = OsString::from("test");
        assert_eq!(from_vec(to_vec(orig.clone())).unwrap(), orig);
    }
}
