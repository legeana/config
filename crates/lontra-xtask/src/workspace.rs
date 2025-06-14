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

pub(crate) fn change_dir_to_root(sh: &Shell) -> Result<()> {
    let workspace_root = root(sh)?;
    eprintln!("$ cd {workspace_root:?}");
    sh.change_dir(workspace_root);
    Ok(())
}

fn verify_root(sh: &Shell, root: &Path) -> Result<()> {
    let workspace_manifest = root.join("Cargo.toml");
    let manifest = sh
        .read_file(&workspace_manifest)
        .with_context(|| format!("failed to read {workspace_manifest:?}"))?;
    if !manifest.contains("[workspace]\n") {
        bail!("{root:?}: Cargo.toml doesn't contain [workspace] section");
    }
    Ok(())
}
