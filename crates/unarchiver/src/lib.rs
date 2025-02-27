mod inventory;
mod unzip;

use std::path::Path;

use anyhow::{Context as _, Result, bail};

pub trait Unarchiver: Send + Sync {
    fn name(&self) -> String;
    fn extensions(&self) -> Vec<String>;
    fn unarchive(&self, src: &Path, dst: &Path) -> Result<()>;
}

type UnarchiverBox = Box<dyn Unarchiver>;

pub fn by_name(name: impl AsRef<str>) -> Result<&'static dyn Unarchiver> {
    inventory::by_name(name.as_ref())
}

pub fn by_filename(path: impl AsRef<Path>) -> Result<&'static dyn Unarchiver> {
    let path = path.as_ref();
    let Some(ext) = path.extension() else {
        bail!("failed to get extension from {path:?}");
    };
    let Some(ext) = ext.to_str() else {
        bail!("failed to convert {ext:?} to utf8");
    };
    inventory::by_extension(ext)
        .with_context(|| format!("failed to find unarchiver for extension {ext:?}"))
}
