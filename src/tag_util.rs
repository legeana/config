use crate::hostname;

use anyhow::{anyhow, Context, Result};

pub fn has_tag(tag: &str) -> Result<bool> {
    match tag.find('=') {
        Some(pos) => has_tag_kv(&tag[..pos], &tag[pos + 1..]),
        None => Err(anyhow!("invalid tag: must contain '=', got {tag}")),
    }
}

fn has_tag_kv(key: &str, value: &str) -> Result<bool> {
    match key {
        "hostname" => match_hostname(value),
        _ => Ok(false),
    }
}

fn match_hostname(want_hostname: &str) -> Result<bool> {
    let got_hostname = hostname::hostname().with_context(|| "unable to get hostname")?;
    Ok(want_hostname == got_hostname)
}
