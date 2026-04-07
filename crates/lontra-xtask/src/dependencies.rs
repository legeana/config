use anyhow::Context as _;
use anyhow::Result;
use xshell::Shell;
use xshell::cmd;

struct Dep {
    package: &'static str,
    want_binary: &'static str,
}

const DEV_DEPS: &[Dep] = &[Dep {
    package: "sqlx-cli",
    want_binary: "sqlx",
}];

pub(crate) fn install_dev(sh: &Shell) -> Result<()> {
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    for dep in DEV_DEPS {
        if which::which(dep.want_binary).is_ok() {
            continue;
        }
        let pkg = dep.package;
        cmd!(sh, "{cargo} install --locked {pkg}").run()?;
    }
    Ok(())
}
