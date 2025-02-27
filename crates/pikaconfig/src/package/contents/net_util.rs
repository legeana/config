use anyhow::{Context as _, Result};

use crate::annotated_path::AnnotatedPath;

use super::file_util;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct FetchOptions {
    executable: bool,
}

impl FetchOptions {
    pub(super) fn new() -> Self {
        Self { executable: false }
    }
    pub(super) fn executable(&mut self, executable: bool) -> &mut Self {
        self.executable = executable;
        self
    }
}

pub(super) fn fetch(
    url: impl AsRef<str>,
    dst: impl AnnotatedPath,
    opts: &FetchOptions,
) -> Result<()> {
    let url = url.as_ref();
    log::info!("Fetch: {url:?} -> {dst:?}");
    let mut reader = ureq::get(url)
        .call()
        .with_context(|| format!("failed to fetch {url:?}"))?
        .into_body()
        .into_reader();
    let output =
        std::fs::File::create(dst.as_path()).with_context(|| format!("failed to open {dst:?}"))?;
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
