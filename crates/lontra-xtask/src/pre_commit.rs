use std::ffi::OsString;

use anyhow::Context as _;
use anyhow::Ok;
use anyhow::Result;
use anyhow::bail;
use xshell::Shell;
use xshell::cmd;

pub fn run() -> Result<()> {
    let sh = Shell::new()?;
    verify_cwd(&sh)?;

    setup_pre_commit(&sh)?;
    fmt_pre_commit(&sh)?;
    sqlx_pre_commit(&sh)?;
    rust_pre_commit(&sh)?;

    Ok(())
}

pub fn verify_cwd(sh: &Shell) -> Result<()> {
    let manifest = sh.read_file("Cargo.toml")?;
    if !manifest.contains("[workspace]\n") {
        bail!("Cargo.toml doesn't contain [workspace] section");
    }
    Ok(())
}

fn setup_pre_commit(sh: &Shell) -> Result<()> {
    cmd!(sh, "./setup -d list").run()?;
    Ok(())
}

fn fmt_pre_commit(sh: &Shell) -> Result<()> {
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    cmd!(sh, "{cargo} fmt --check").run()?;
    Ok(())
}

fn sqlx_pre_commit(sh: &Shell) -> Result<()> {
    let sh = sh.clone(); // Shell uses interior mutability.
    // Set DATABASE_URL.
    let db = sh.current_dir().join("target").join("sqlx.sqlite");
    let url = {
        let mut url = OsString::from("sqlite://");
        url.push(db);
        url
    };
    sh.set_var("DATABASE_URL", url);

    sh.change_dir("crates/lontra-registry");
    cmd!(sh, "sqlx database reset -y").run()?;
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    cmd!(
        sh,
        "{cargo} sqlx prepare --check -- --all-targets --all-features"
    )
    .run()?;

    Ok(())
}

fn rust_targets(sh: &Shell) -> Result<Vec<String>> {
    let targets = cmd!(sh, "rustup target list --installed").read()?;
    Ok(targets.split_ascii_whitespace().map(String::from).collect())
}

fn rust_pre_commit(sh: &Shell) -> Result<()> {
    let cargo_args = ["--release", "--all-targets"];
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    let has_xwin = cmd!(sh, "{cargo} xwin --version")
        .ignore_stdout()
        .ignore_stderr()
        .run()
        .is_ok();
    if has_xwin {
        for target in rust_targets(sh)? {
            cmd!(sh, "{cargo} xwin check --target={target} {cargo_args...}").run()?;
            // Treat warnings as errors.
            cmd!(
                sh,
                "{cargo} xwin clippy --target={target} {cargo_args...} -- -Dwarnings"
            )
            .run()?;
            cmd!(sh, "{cargo} xwin test --target={target} {cargo_args...}").run()?;
        }
    } else {
        cmd!(sh, "{cargo} check {cargo_args...}").run()?;
        // Treat warnings as errors.
        cmd!(sh, "{cargo} clippy {cargo_args...} -- -Dwarnings").run()?;
        cmd!(sh, "{cargo} test {cargo_args...}").run()?;
    }
    Ok(())
}
