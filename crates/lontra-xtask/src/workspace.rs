use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::Ok;
use anyhow::Result;
use anyhow::bail;
use xshell::Shell;
use xshell::cmd;

pub(crate) fn root(sh: &Shell) -> Result<PathBuf> {
    let cargo = sh.var_os("CARGO").context("failed to find CARGO")?;
    let workspace_manifest: PathBuf = cmd!(
        sh,
        "{cargo} locate-project --workspace --message-format=plain"
    )
    .read()?
    .into();
    let workspace_root = workspace_manifest
        .parent()
        .with_context(|| format!("failed to find {workspace_manifest:?}'s parent"))?;
    verify_root(sh, workspace_root)?;
    Ok(workspace_root.into())
}

pub(crate) fn crate_root(sh: &Shell, crate_: &str) -> Result<PathBuf> {
    let workspace_root = root(sh)?;
    // Assuming lontra project layout.
    // This may not work in other projects.
    let crate_root = workspace_root.join("crates").join(crate_);
    verify_crate(sh, crate_, &crate_root)?;
    Ok(crate_root)
}

pub(crate) fn change_dir_to_root(sh: &Shell) -> Result<()> {
    let workspace_root = root(sh)?;
    eprintln!("$ cd {workspace_root:?}");
    sh.change_dir(workspace_root);
    Ok(())
}

pub(crate) fn change_dir_to_crate(sh: &Shell, crate_: &str) -> Result<()> {
    let crate_root = crate_root(sh, crate_)?;
    eprintln!("$ cd {crate_root:?}");
    sh.change_dir(crate_root);
    Ok(())
}

fn verify_root(sh: &Shell, root: &Path) -> Result<()> {
    let workspace_manifest = root.join("Cargo.toml");
    let manifest = sh
        .read_file(&workspace_manifest)
        .with_context(|| format!("failed to read {workspace_manifest:?}"))?;
    let expected_entry = "[workspace]";
    if !manifest.contains(expected_entry) {
        bail!("{root:?}: Cargo.toml doesn't contain: {expected_entry}");
    }
    Ok(())
}

fn verify_crate(sh: &Shell, crate_: &str, crate_root: &Path) -> Result<()> {
    let crate_manifest = crate_root.join("Cargo.toml");
    let manifest = sh
        .read_file(&crate_manifest)
        .with_context(|| format!("failed to read {crate_manifest:?}"))?;
    let expected_entry = format!("name = {crate_:?}");
    if !manifest.contains(&expected_entry) {
        bail!("{crate_root:?}: Cargo.toml doesn't contain: {expected_entry}");
    }
    Ok(())
}
