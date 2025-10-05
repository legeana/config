use anyhow::Context as _;
use anyhow::Result;
use xshell::Shell;
use xshell::cmd;

const DEV_DEPS: &[&str] = &["cargo-xwin", "sqlx-cli"];

pub fn install_dev() -> Result<()> {
    let sh = Shell::new()?;
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    cmd!(sh, "{cargo} install --locked {DEV_DEPS...}").run()?;
    Ok(())
}
