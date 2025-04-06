use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct Context {
    // Filename of the template.
    pub(crate) source_file: PathBuf,
    // Directory of the template.
    pub(crate) source_dir: PathBuf,
    // Filename of the rendered file.
    pub(crate) destination_file: PathBuf,
    // Directory of the rendered file.
    pub(crate) destination_dir: PathBuf,
    // Directory of MANIFEST.
    // May be different from source_dir if render argument consists of multiple
    // path components.
    pub(crate) workdir: PathBuf,
    // MANIFEST prefix render was called in.
    // May be different from destination_dir if render_to is used.
    pub(crate) prefix: PathBuf,
}
