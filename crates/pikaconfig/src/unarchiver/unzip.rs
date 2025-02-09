use std::path::Path;

use anyhow::Result;

struct Unzip;

impl super::Unarchiver for Unzip {
    fn name(&self) -> String {
        "unzip".to_owned()
    }
    fn extensions(&self) -> Vec<String> {
        vec!["zip".to_owned()]
    }
    fn unarchive(&self, src: &Path, dst: &Path) -> Result<()> {
        process_utils::run_verbose(
            std::process::Command::new("unzip")
                .arg("-o")
                .arg("-q")
                .arg(src)
                .arg("-d")
                .arg(dst),
        )
    }
}

pub fn register(registry: &mut super::inventory::Registry) {
    registry.register(Box::new(Unzip))
}
