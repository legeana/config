use anyhow::Context as _;
use anyhow::Ok;
use anyhow::Result;
use anyhow::anyhow;
use xshell::Shell;
use xshell::cmd;

use crate::install::install_shim;
use crate::workspace;

pub fn run() -> Result<()> {
    let sh = Shell::new()?;
    workspace::change_dir_to_root(&sh)?;

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

fn rust_targets(sh: &Shell) -> Result<Vec<String>> {
    let targets = cmd!(sh, "rustup target list --installed").read()?;
    Ok(targets.split_ascii_whitespace().map(String::from).collect())
}

fn argv_from_cmd<'a>(cmd: &'a str, args: &[&'a str]) -> impl Iterator<Item = &'a str> {
    std::iter::once(cmd).chain(args.iter().copied())
}

fn find_wine(sh: &Shell) -> Result<String> {
    const WINES: &[(&str, &[&str])] = &[
        ("wine", &[]),
        ("flatpak", &["run", "--filesystem=home", "org.winehq.Wine"]),
    ];
    for (wine, wine_args) in WINES {
        let wine = *wine;
        let wine_args = *wine_args;
        let argv = argv_from_cmd(wine, wine_args);
        if cmd!(sh, "{wine} {wine_args...} --version").run().is_ok() {
            return Ok(itertools::join(argv, " "));
        }
    }
    Err(anyhow!("wine not found"))
}

fn xwin_target_env(rust_target: &str) -> String {
    let xwin_target = rust_target.to_uppercase().replace('-', "_");
    format!("CARGO_TARGET_{xwin_target}_RUNNER")
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
        let wine = find_wine(sh)?;
        for target in rust_targets(sh)? {
            let sh = sh.clone();
            if target.contains("-windows-") {
                sh.set_var(xwin_target_env(&target), &wine);
            }
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
