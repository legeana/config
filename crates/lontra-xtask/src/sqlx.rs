use std::ffi::OsString;

use anyhow::Context as _;
use anyhow::Result;
use xshell::Shell;
use xshell::cmd;

use crate::workspace;

const REGISTRY_CRATE: &str = "lontra-registry";

pub fn prepare() -> Result<()> {
    let sh = Shell::new()?;
    set_database_url(&sh)?;
    workspace::change_dir_to_crate(&sh, REGISTRY_CRATE)?;

    cmd!(sh, "sqlx database reset -y").run()?;
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    cmd!(sh, "{cargo} sqlx prepare -- --all-targets --all-features").run()?;

    Ok(())
}

pub(crate) fn pre_commit(sh: &Shell) -> Result<()> {
    let sh = sh.clone(); // Shell uses interior mutability.
    set_database_url(&sh)?;
    workspace::change_dir_to_crate(&sh, REGISTRY_CRATE)?;
    // Verify sqlx offline files.
    cmd!(sh, "sqlx database reset -y").run()?;
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    cmd!(
        sh,
        "{cargo} sqlx prepare --check -- --all-targets --all-features"
    )
    .run()?;

    Ok(())
}

fn set_database_url(sh: &Shell) -> Result<()> {
    let workspace_root = workspace::root(sh)?;
    let db = workspace_root.join("target").join("sqlx.sqlite");
    let url = {
        // https://github.com/launchbadge/sqlx/issues/2771#issuecomment-2831396223
        let mut url = OsString::from("sqlite:");
        url.push(db);
        url
    };
    eprintln!("DATABASE_URL={}", url.to_string_lossy());
    sh.set_var("DATABASE_URL", url);

    Ok(())
}
