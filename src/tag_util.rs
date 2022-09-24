use anyhow::{anyhow, Context, Result};
use sysinfo::{System, SystemExt};

pub fn has_tag(tag: &str) -> Result<bool> {
    match tag.find('=') {
        Some(pos) => has_tag_kv(&tag[..pos], &tag[pos + 1..]),
        None => Err(anyhow!("invalid tag: must contain '=', got {tag}")),
    }
}

pub fn has_all_tags<T: AsRef<str>>(tags: &[T]) -> Result<bool> {
    for tag in tags {
        let tag = tag.as_ref();
        let has = !has_tag(tag).with_context(|| format!("failed to check tag {tag:?}"))?;
        if !has {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn has_any_tags<T: AsRef<str>>(tags: &[T]) -> Result<bool> {
    for tag in tags {
        let tag = tag.as_ref();
        let has = !has_tag(tag).with_context(|| format!("failed to check tag {tag:?}"))?;
        if has {
            return Ok(true);
        }
    }
    Ok(false)
}

fn has_tag_kv(key: &str, value: &str) -> Result<bool> {
    match key {
        "distro" => match_distro(value),
        "family" => match_family(value),
        "hostname" => match_hostname(value),
        "os" => match_os(value),
        _ => Ok(false),
    }
}

/// Returns system tags.
pub fn tags() -> Result<Vec<String>> {
    Ok(vec![
        format!("distro={}", distro()?),
        format!("hostname={}", hostname()?),
        format!("family={}", family()),
        format!("os={}", os()),
    ])
}

/// Returns 'windows' or 'unix'.
fn family() -> &'static str {
    std::env::consts::FAMILY
}

fn match_family(want_family: &str) -> Result<bool> {
    Ok(want_family == family())
}

fn hostname() -> Result<String> {
    let sys = System::new();
    sys.host_name()
        .ok_or_else(|| anyhow!("failed to obtain hostname"))
}

fn match_hostname(want_hostname: &str) -> Result<bool> {
    Ok(want_hostname == hostname()?)
}

/// Returns 'linux', 'macos', 'windows' etc.
/// See https://doc.rust-lang.org/std/env/consts/constant.OS.html
fn os() -> &'static str {
    std::env::consts::OS
}

fn match_os(want_os: &str) -> Result<bool> {
    Ok(want_os == os())
}

fn distro() -> Result<String> {
    let sys = System::new();
    sys.name()
        .ok_or_else(|| anyhow!("failed to obtain distro"))
}

fn match_distro(want_distro: &str) -> Result<bool> {
    Ok(want_distro == distro()?)
}
