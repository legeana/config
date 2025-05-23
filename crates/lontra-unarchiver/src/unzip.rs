use std::path::Path;

use anyhow::Result;
use lontra_process::cmd;

struct Unzip;

impl super::Unarchiver for Unzip {
    fn name(&self) -> String {
        "unzip".to_owned()
    }
    fn extensions(&self) -> Vec<String> {
        vec!["zip".to_owned()]
    }
    fn unarchive(&self, src: &Path, dst: &Path) -> Result<()> {
        cmd!(["unzip", "-o", "-q", src, "-d", dst]).run_verbose()
    }
}

pub(crate) fn register(registry: &mut super::inventory::Registry) {
    registry.register(Box::new(Unzip));
}
