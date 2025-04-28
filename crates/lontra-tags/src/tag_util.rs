use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context as _, Result, anyhow};
use sysinfo::System;

static SYSINFO: LazyLock<SystemInfo> = LazyLock::new(SystemInfo::new);

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

fn has_tag_kv(key: &str, value: &str) -> bool {
    match key {
        "feature" => SYSINFO.match_feature(value),
        "distro" => SYSINFO.match_distro(value),
        "distro_like" => SYSINFO.match_distro_like(value),
        "family" => SYSINFO.match_family(value),
        "hostname" => SYSINFO.match_hostname(value),
        "os" => SYSINFO.match_os(value),
        "uid" => SYSINFO.match_uid(value),
        _ => false,
    }
}

#[cfg(unix)]
fn getuid() -> Option<u32> {
    Some(unsafe { libc::getuid() })
}

#[cfg(windows)]
fn getuid() -> Option<u32> {
    None
}

struct SystemInfo;

impl SystemInfo {
    fn new() -> Self {
        Self {}
    }
    fn is_wsl(&self) -> bool {
        std::env::var("WSL_DISTRO_NAME").is_ok()
    }
    fn match_feature(&self, want_feature: &str) -> bool {
        match want_feature {
            "wsl" => self.is_wsl(),
            _ => false,
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
        System::host_name()
    }
    fn match_hostname(&self, want_hostname: &str) -> bool {
        Some(want_hostname.into()) == self.hostname()
    }
    /// Returns 'linux', 'macos', 'windows' etc.
    /// See <https://doc.rust-lang.org/std/env/consts/constant.OS.html>.
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
        System::distribution_id()
    }
    fn match_distro(&self, want_distro: &str) -> bool {
        want_distro == self.distro()
    }
    fn distro_like(&self) -> Vec<String> {
        let id = System::distribution_id();
        let mut id_like = System::distribution_id_like();
        if !id_like.contains(&id) {
            id_like.insert(0, id);
        }
        id_like
    }
    fn match_distro_like(&self, want_distro: &str) -> bool {
        self.distro_like().contains(&want_distro.to_owned())
    }
    fn match_uid(&self, want_uid: &str) -> bool {
        match getuid() {
            Some(uid) => uid.to_string() == want_uid,
            None => false,
        }
    }
}

/// Returns system tags.
pub fn tags() -> Result<Vec<String>> {
    // Always present.
    let mut tags = vec![
        format!("distro={}", SYSINFO.distro()),
        format!(
            "hostname={}",
            SYSINFO.hostname().unwrap_or_else(|| "N/A".into())
        ),
        format!("family={}", SYSINFO.family()),
        format!("os={}", SYSINFO.os()),
    ];
    // Multiple distro_like can be present at the same time.
    for distro_like in SYSINFO.distro_like() {
        tags.push(format!("distro_like={distro_like}"));
    }
    // Multiple features can be present at the same time.
    for feature in ["wsl"] {
        if SYSINFO.match_feature(feature) {
            tags.push(format!("feature={feature}"));
        }
    }
    // Miscellaneous.
    if let Some(uid) = getuid() {
        tags.push(format!("uid={uid}"));
    }
    // Return sorted tags.
    tags.sort();
    Ok(tags)
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
