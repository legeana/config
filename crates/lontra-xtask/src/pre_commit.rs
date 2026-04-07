use anyhow::Context as _;
use anyhow::Ok;
use anyhow::Result;
use xshell::Shell;
use xshell::cmd;

use crate::dependencies::install_dev;
use crate::install::install_shim;
use crate::workspace;

pub fn run() -> Result<()> {
    let sh = Shell::new()?;
    workspace::change_dir_to_root(&sh)?;

    install_dev(&sh)?;
    setup_pre_commit(&sh)?;
    fmt_pre_commit(&sh)?;
    crate::sqlx::pre_commit(&sh)?;
    rust_pre_commit(&sh)?;

    Ok(())
}

pub fn install() -> Result<()> {
    static SHIM: &str = "pre-commit-shim";
    let sh = Shell::new()?;
    // Why shims?
    // - Shims are trivial and don't need to be rebuilt frequently.
    // - Symlinks are not always available on Windows, and even if they are we
    //   need to store the executable somewhere (e.g. in target). But in that
    //   case the hook will fail if we run `cargo clean`.
    install_shim(&sh, SHIM).with_context(|| format!("failed to install {SHIM}"))
}

fn setup_pre_commit(sh: &Shell) -> Result<()> {
    if cfg!(windows) {
        cmd!(sh, "./setup.bat -d list").run()?;
    } else {
        cmd!(sh, "./setup -d list").run()?;
    }
    Ok(())
}

fn fmt_pre_commit(sh: &Shell) -> Result<()> {
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    cmd!(sh, "{cargo} fmt --check").run()?;
    Ok(())
}

fn rust_pre_commit(sh: &Shell) -> Result<()> {
    let cargo_args = ["--release", "--all-targets"];
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    cmd!(sh, "{cargo} check {cargo_args...}").run()?;
    // Treat warnings as errors.
    cmd!(sh, "{cargo} clippy {cargo_args...} -- -Dwarnings").run()?;
    cmd!(sh, "{cargo} test {cargo_args...}").run()?;
    Ok(())
}
