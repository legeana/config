use anyhow::Context as _;
use anyhow::Result;
use xshell::Shell;
use xshell::cmd;

const DEV_DEPS: &[&str] = &["cargo-xwin", "sqlx-cli"];
const DEV_TARGETS: &[&str] = &["x86_64-pc-windows-gnu", "x86_64-unknown-linux-gnu"];

pub fn install_dev() -> Result<()> {
    let sh = Shell::new()?;
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    cmd!(sh, "{cargo} install --locked {DEV_DEPS...}").run()?;
    cmd!(sh, "rustup target add {DEV_TARGETS...}").run()?;
    Ok(())
}
