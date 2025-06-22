use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq)]
pub struct RawAllowedKey {
    // See ssh-keygen(1) ALLOWED SIGNERS.
    pub principals: &'static str,
    pub ssh_key: &'static str,
}

static BUILTIN: LazyLock<&[RawAllowedKey]> = LazyLock::new(|| {
    // TODO: load from file, e.g. via include_str!().
    &[RawAllowedKey {
        principals: "legeana@liri.ie",
        ssh_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDDShKKJSxIoOefearxLMuKT+Y4TkyypOTU4weoatzvJ",
    }]
});

pub(crate) fn builtin() -> &'static [RawAllowedKey] {
    *BUILTIN
}
