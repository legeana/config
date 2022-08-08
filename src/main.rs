mod layout;
mod repository;

use anyhow::Result;
use anyhow::anyhow;

use std::env;
use std::path::PathBuf;

fn config_root() -> Result<PathBuf> {
    let exe_path = env::current_exe()?;
    let mut parent = exe_path.parent();
    while let Some(dir) = parent {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(dir.to_path_buf());
        }
        parent = dir.parent();
    }
    Err(anyhow!("unable to find Cargo.toml, use setup helper"))
}

fn main() -> Result<()> {
    let root = config_root()?;
    println!("Found user configuration: {}", root.display());
    let _repos = layout::repositories(&root)?;
    Ok(())
}
