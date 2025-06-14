use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::Result;
use xshell::Shell;
use xshell::cmd;

fn maybe_remove_file(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        r @ Ok(()) => r,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        },
    }
}

fn git_root(sh: &Shell) -> Result<PathBuf> {
    let root = cmd!(sh, "git rev-parse --show-toplevel").read()?;
    Ok(PathBuf::from(root))
}

pub fn install_shim(sh: &Shell, shim: &str) -> Result<()> {
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    let pkg = env!("CARGO_PKG_NAME");
    cmd!(sh, "{cargo} run --package={pkg} --bin={shim} -- --install").run()?;
    Ok(())
}

pub fn install_self_as_git_hook(sh: &Shell, hook: &str) -> Result<()> {
    let src = std::env::current_exe()?;
    let dst = git_root(sh)?
        .join(".git")
        .join("hooks")
        .join(hook)
        .with_extension(std::env::consts::EXE_EXTENSION);
    maybe_remove_file(&dst)?;
    std::fs::copy(src, dst)?;
    Ok(())
}
