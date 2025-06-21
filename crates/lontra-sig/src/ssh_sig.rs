#![allow(clippy::needless_continue)]

use std::sync::LazyLock;

use ssh_key::PublicKey;
use ssh_key::SshSig;

#[derive(Clone, Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("invalid ssh key {0:?}: {1}")]
    InvalidSshKey(RawAllowedKey, ssh_key::Error),
    #[error("invalid signature: {0}")]
    InvalidSignature(ssh_key::Error),
    #[error("invalid namespace {0}, expected {NAMESPACE}")]
    InvalidNamespace(String),
    #[error("the signature is valid but not trusted")]
    UntrustedSignature,
    #[error(
        "found key {0:?} matching the signature's public key, but the verification failed: {1}"
    )]
    CryptographicError(RawAllowedKey, ssh_key::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

const NAMESPACE: &str = "lontra";

#[derive(Clone, Debug, PartialEq)]
pub struct RawAllowedKey {
    // See ssh-keygen(1) ALLOWED SIGNERS.
    principals: &'static str,
    ssh_key: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
struct AllowedKey {
    raw_key: RawAllowedKey,
    ssh_key: PublicKey,
}

#[derive(Debug)]
struct AllowedKeySet {
    keys: Vec<AllowedKey>,
}

impl AllowedKeySet {
    fn from_raw_keys(raw_keys: &[RawAllowedKey]) -> Result<Self> {
        let mut r = Self { keys: Vec::new() };
        for key in raw_keys {
            let ssh_key = PublicKey::from_openssh(key.ssh_key)
                .map_err(|err| Error::InvalidSshKey(key.clone(), err))?;
            r.keys.push(AllowedKey {
                raw_key: key.clone(),
                ssh_key,
            });
        }
        Ok(r)
    }
    fn iter(&self) -> std::slice::Iter<'_, AllowedKey> {
        self.keys.iter()
    }
    fn verify(&self, msg: impl AsRef<[u8]>, ssh_sig: impl AsRef<[u8]>) -> Result<()> {
        let sig = SshSig::from_pem(ssh_sig).map_err(Error::InvalidSignature)?;

        for allowed_key in self.iter() {
            match allowed_key.ssh_key.verify(NAMESPACE, msg.as_ref(), &sig) {
                Ok(()) => return Ok(()),
                Err(ssh_key::Error::Namespace) => {
                    // Namespace is a property of the signature, so retrying
                    // with a different key won't help. Returning early to
                    // produce a better error message.
                    return Err(Error::InvalidNamespace(sig.namespace().to_owned()));
                }
                Err(ssh_key::Error::PublicKey) => {
                    // Other keys may still match.
                    continue;
                }
                Err(e) => {
                    return Err(Error::CryptographicError(allowed_key.raw_key.clone(), e));
                }
            };
        }

        // Fail-close.
        Err(Error::UntrustedSignature)
    }
}

// TODO: load from file, e.g. via include_str!().
static RAW_ALLOWED_KEYS: &[RawAllowedKey] = &[RawAllowedKey {
    principals: "legeana@liri.ie",
    ssh_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDDShKKJSxIoOefearxLMuKT+Y4TkyypOTU4weoatzvJ",
}];
static ALLOWED_KEYS: LazyLock<AllowedKeySet> = LazyLock::new(|| {
    AllowedKeySet::from_raw_keys(RAW_ALLOWED_KEYS).expect("failed to parse RAW_ALLOWED_KEYS")
});

/// Verifies that a given message is signed by a given ssh signature, and the
/// signature is trusted by lontra.
///
/// The list of trusted keys is compiled into the binary.
pub fn verify(msg: impl AsRef<[u8]>, ssh_sig: impl AsRef<[u8]>) -> Result<()> {
    ALLOWED_KEYS.verify(msg, ssh_sig)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::assert_matches;

    use super::*;

    // ALLOWED_KEYS may change in the future making tests brittle.
    // Use test-only values instead.
    const TEST_RAW_ALLOWED_KEYS: &[RawAllowedKey] = &[RawAllowedKey {
        principals: "a@example.com",
        ssh_key: include_str!("../testdata/trusted_a_id_ed25519.pub"),
    }];
    static TEST_ALLOWED_KEYS: LazyLock<AllowedKeySet> = LazyLock::new(|| {
        AllowedKeySet::from_raw_keys(TEST_RAW_ALLOWED_KEYS).expect("invalid TEST_RAW_ALLOWED_KEYS")
    });
    // Precompute a known valid msg/signature pair. These values are used
    // exclusively for testing and have no bearing on signing.
    const TEST_MSG: &str = "test message";
    // A signature listed in TEST_ALLOWED_KEYS with namespace=NAMESPACE.
    const TEST_GOOD_SIG: &str = include_str!("../testdata/trusted-a@lontra.sig");
    // A signature that can't be decoded.
    const TEST_INVALID_SIG: &str = r"
-----BEGIN SSH SIGNATURE-----
INVALID
-----END SSH SIGNATURE-----";
    // A truncated signature.
    const TEST_TRUNCATED_SIG: &str = r"
-----BEGIN SSH SIGNATURE-----
U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAg9LrBUjaWAah9Rj7MjjM0TK1NgL
61
-----END SSH SIGNATURE-----";
    // A signature listed in TEST_ALLOWED_KEYS with namespace="bad".
    const TEST_BAD_NAMESPACE_SIG: &str = include_str!("../testdata/trusted@bad.sig");
    // A signature not listed in TEST_ALLOWED_KEYS with namespace=NAMESPACE.
    const TEST_UNTRUSTED_SIG: &str = include_str!("../testdata/untrusted@lontra.sig");

    #[test]
    fn test_allowed_keys_not_empty() {
        // A simple sanity check.
        let count = ALLOWED_KEYS.iter().count();
        assert!(count > 0, "count > 0, count = {count}");
    }

    #[test]
    fn test_verify() {
        TEST_ALLOWED_KEYS
            .verify(TEST_MSG, TEST_GOOD_SIG)
            .expect("trusted signature");
    }

    #[test_case(TEST_INVALID_SIG)]
    #[test_case(TEST_TRUNCATED_SIG)]
    fn test_verify_invalid_sig(invalid_sig: &str) {
        let r = TEST_ALLOWED_KEYS.verify(TEST_MSG, invalid_sig);
        assert_matches!(r, Err(Error::InvalidSignature(_)));
    }

    #[test]
    fn test_verify_bad_namespace() {
        let r = TEST_ALLOWED_KEYS.verify(TEST_MSG, TEST_BAD_NAMESPACE_SIG);
        assert_eq!(r, Err(Error::InvalidNamespace("bad".to_owned())));
    }

    #[test]
    fn test_verify_untrusted() {
        let r = TEST_ALLOWED_KEYS.verify(TEST_MSG, TEST_UNTRUSTED_SIG);
        assert_eq!(r, Err(Error::UntrustedSignature));
    }

    #[test]
    fn test_verify_cryptographic() {
        let r = TEST_ALLOWED_KEYS.verify("wrong-message", TEST_GOOD_SIG);
        assert_matches!(r, Err(Error::CryptographicError(_, _)));
    }
}
