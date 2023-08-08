use std::path::Path;

use anyhow::{Context, Result};

use super::file_util;

#[derive(Clone, Debug, PartialEq)]
pub struct FetchOptions {
    executable: bool,
}

impl FetchOptions {
    pub fn new() -> Self {
        Self { executable: false }
    }
    pub fn executable(&mut self, executable: bool) -> &mut Self {
        self.executable = executable;
        self
    }
}

pub fn fetch(url: impl AsRef<str>, dst: impl AsRef<Path>, opts: &FetchOptions) -> Result<()> {
    let url = url.as_ref();
    let dst = dst.as_ref();
    let mut reader = ureq::get(url)
        .call()
        .with_context(|| format!("failed to fetch {url:?}"))?
        .into_reader();
    let output = std::fs::File::create(dst).with_context(|| format!("failed to open {dst:?}"))?;
    let mut writer = std::io::BufWriter::new(&output);
    std::io::copy(&mut reader, &mut writer).with_context(|| format!("failed to write {dst:?}"))?;
    if opts.executable {
        file_util::set_file_executable(&output)
            .with_context(|| format!("failed to make {dst:?} executable"))?;
    }
    output
        .sync_all()
        .with_context(|| format!("failed to flush {dst:?}"))
}
