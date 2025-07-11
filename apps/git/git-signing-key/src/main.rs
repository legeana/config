use std::fmt;

use anyhow::bail;
use anyhow::Context as _;
use anyhow::Result;
use clap::Parser;
use xshell::cmd;
use xshell::Shell;

#[derive(Debug, Parser)]
struct Cli;

#[derive(PartialEq)]
struct SshKey {
    kind: String,
    pubkey: String,
}

impl fmt::Debug for SshKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for SshKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.kind, self.pubkey)
    }
}

fn main() -> Result<()> {
    let _args = Cli::parse();
    let sh = Shell::new()?;

    let available = available_keys(&sh)?;
    let allowed = allowed_keys(&sh)?;
    for a in &allowed {
        if available.contains(a) {
            println!("key::{a}");
            return Ok(())
        }
    }

    bail!("failed to find allowed signing key, allowed = {allowed:?}, available = {available:?}");
}

fn line_fields(line: &str, a: usize, b: usize) -> Option<(&str, &str)> {
    let split = line.trim().split_ascii_whitespace();
    let mut a_s = None;
    let mut b_s = None;
    for (i, part) in split.enumerate() {
        if i == a {
            a_s = Some(part);
        }
        if i == b {
            b_s = Some(part);
        }
        if let Some(a_s) = a_s
            && let Some(b_s) = b_s
        {
            return Some((a_s, b_s));
        }
    }
    None
}

fn split_ssh_add_line(line: &str) -> Result<SshKey> {
    let (kind, pubkey) = line_fields(line, 0, 1)
        .with_context(|| format!("failed to split ssh-add output: {line}"))?;
    Ok(SshKey {
        kind: kind.to_owned(),
        pubkey: pubkey.to_owned(),
    })
}

fn split_allowed_signers_line(line: &str) -> Result<SshKey> {
    let (kind, pubkey) = line_fields(line, 1, 2)
        .with_context(|| format!("failed to split allowed-signers line: {line}"))?;
    Ok(SshKey {
        kind: kind.to_owned(),
        pubkey: pubkey.to_owned(),
    })
}

fn available_keys(sh: &Shell) -> Result<Vec<SshKey>> {
    let output = cmd!(sh, "ssh-add -L").quiet().read()?;
    let keys = output
        .lines()
        .map(split_ssh_add_line)
        .collect::<Result<_>>()?;
    Ok(keys)
}

fn allowed_keys(sh: &Shell) -> Result<Vec<SshKey>> {
    let output = cmd!(sh, "git config get gpg.ssh.allowedSignersFile").quiet().read()?;
    let path = shellexpand::path::tilde(&output);
    let allowed = sh.read_file(&path)?;
    let keys = allowed
        .lines()
        .map(split_allowed_signers_line)
        .collect::<Result<_>>()?;
    Ok(keys)
}
