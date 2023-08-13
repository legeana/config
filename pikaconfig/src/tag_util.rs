use std::path::Path;

use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use sysinfo::{System, SystemExt};

static SYSINFO: Lazy<SystemInfo> = Lazy::new(SystemInfo::new);

pub fn has_tag(tag: &str) -> Result<bool> {
    if let Some((key, value)) = tag.split_once("!=") {
        return Ok(!has_tag_kv(key, value));
    }
    match tag.split_once('=') {
        Some((key, value)) => Ok(has_tag_kv(key, value)),
        None => Err(anyhow!("invalid tag: must contain '=', got {tag}")),
    }
}

pub fn has_all_tags<T: AsRef<str>>(tags: &[T]) -> Result<bool> {
    for tag in tags {
        let tag = tag.as_ref();
        let has = has_tag(tag).with_context(|| format!("failed to check tag {tag:?}"))?;
        if !has {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn has_any_tags<T: AsRef<str>>(tags: &[T]) -> Result<bool> {
    for tag in tags {
        let tag = tag.as_ref();
        let has = has_tag(tag).with_context(|| format!("failed to check tag {tag:?}"))?;
        if has {
            return Ok(true);
        }
    }
    Ok(false)
}

fn has_tag_kv(key: &str, value: &str) -> bool {
    match key {
        "distro" => SYSINFO.match_distro(value),
        "family" => SYSINFO.match_family(value),
        "hostname" => SYSINFO.match_hostname(value),
        "os" => SYSINFO.match_os(value),
        "uid" => SYSINFO.match_uid(value),
        _ => false,
    }
}

struct SystemInfo {
    system: System,
}

impl SystemInfo {
    fn new() -> Self {
        Self {
            system: System::new(),
        }
    }
    /// Returns 'windows' or 'unix'.
    fn family(&self) -> &'static str {
        std::env::consts::FAMILY
    }
    fn match_family(&self, want_family: &str) -> bool {
        want_family == self.family()
    }
    fn hostname(&self) -> Option<String> {
        self.system.host_name()
    }
    fn match_hostname(&self, want_hostname: &str) -> bool {
        Some(want_hostname.into()) == self.hostname()
    }
    /// Returns 'linux', 'macos', 'windows' etc.
    /// See https://doc.rust-lang.org/std/env/consts/constant.OS.html
    fn os(&self) -> &'static str {
        std::env::consts::OS
    }
    fn match_os(&self, want_os: &str) -> bool {
        want_os == self.os()
    }
    fn is_unraid(&self) -> bool {
        if self.os() != "linux" {
            // Unraid is linux.
            return false;
        }
        // Unraid's /etc/os-release#ID is "slackware".
        // It's not particularly useful because Unraid is not a general-purpose
        // distro. Instead we check if /etc/unraid-version exists.
        let unraid_version = Path::new("/etc/unraid-version");
        unraid_version.exists()
    }
    fn distro(&self) -> String {
        if self.is_unraid() {
            return "unraid".to_owned();
        }
        self.system.distribution_id()
    }
    fn match_distro(&self, want_distro: &str) -> bool {
        want_distro == self.distro()
    }
    #[cfg(unix)]
    fn match_uid(&self, want_uid: &str) -> bool {
        let uid = unsafe { libc::getuid() };
        uid.to_string() == want_uid
    }
    #[cfg(windows)]
    fn match_uid(&self, _want_uid: &str) -> bool {
        false
    }
}

/// Returns system tags.
pub fn tags() -> Result<Vec<String>> {
    Ok(vec![
        format!("distro={}", SYSINFO.distro()),
        format!(
            "hostname={}",
            SYSINFO.hostname().unwrap_or_else(|| "N/A".into())
        ),
        format!("family={}", SYSINFO.family()),
        format!("os={}", SYSINFO.os()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tags() {
        let tags = tags().expect("tags()");
        assert!(!tags.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn test_family() {
        assert!(has_tag("family=unix").unwrap());
        assert!(!has_tag("family!=unix").unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn test_family() {
        assert!(has_tag("family=windows").unwrap());
        assert!(!has_tag("family!=windows").unwrap());
    }
}
