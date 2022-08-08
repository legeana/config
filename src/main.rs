use std::env;
use std::result;
use std::path::PathBuf;

type Result<T> = result::Result<T, String>;

fn config_root() -> Result<PathBuf> {
    let exe_path = match env::current_exe() {
        Ok(path) => Ok(path),
        Err(err) => Err(err.to_string()),
    }?;
    let mut parent = exe_path.parent();
    while let Some(dir) = parent {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(dir.to_path_buf());
        }
        parent = dir.parent();
    }
    Err("unable to find Cargo.toml, use setup helper".to_string())
}

fn main() -> Result<()> {
    let root = config_root()?;
    println!("Found user configuration: {}", root.display());
    Ok(())
}
