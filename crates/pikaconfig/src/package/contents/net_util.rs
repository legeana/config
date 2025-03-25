use anyhow::{Context as _, Result};
use http::Uri as HttpUri;

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

#[derive(Clone)]
pub(super) struct Url {
    text: String,
    uri: HttpUri,
}

impl std::fmt::Debug for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.text, f)
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.text, f)
    }
}

impl Url {
    pub(super) fn new(text_url: impl Into<String>) -> Result<Self> {
        let text: String = text_url.into();
        let uri = text
            .parse::<HttpUri>()
            .with_context(|| format!("failed to parse URL {text:?}"))?;
        Ok(Self { text, uri })
    }
    pub(super) fn text(&self) -> &str {
        &self.text
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        self.text()
    }
}

pub(super) fn fetch(url: &Url, dst: impl AnnotatedPath, opts: &FetchOptions) -> Result<()> {
    log::info!("Fetch: {url:?} -> {dst:?}");
    let mut reader = ureq::get(&url.uri)
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
