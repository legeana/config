#![allow(dead_code)]
use std::process::Command;

use anyhow::{anyhow, Context, Error, Result};
use xshell::{Cmd, Shell};

use crate::shlexfmt;

fn pretty_args(cmd: &Command) -> String {
    let mut result: Vec<String> = Vec::new();
    result.push(cmd.get_program().to_string_lossy().to_string());
    for arg in cmd.get_args() {
        result.push(arg.to_string_lossy().to_string());
    }
    shlexfmt::join(result.iter().map(String::as_str))
}

fn pretty_print(cmd: &Command) -> String {
    let mut result: Vec<String> = Vec::new();
    if let Some(current_dir) = cmd.get_current_dir() {
        result.push(shlexfmt::quote(&current_dir.to_string_lossy()).to_string());
    }
    result.push("$".to_owned());
    result.push(pretty_args(cmd));
    result.join(" ")
}

fn pretty_print_xs(sh: &Shell, cmd: &Cmd) -> String {
    let current_dir = std::env::current_dir().ok();
    let mut result: Vec<String> = Vec::new();
    if Some(sh.current_dir()) != current_dir {
        result.push(shlexfmt::quote(&sh.current_dir().to_string_lossy()).to_string());
    }
    result.push("$".to_owned());
    result.push(cmd.to_string());
    result.join(" ")
}

fn pretty_err(err: std::io::Error, cmd: &Command) -> Error {
    let cmd = cmd.get_program();
    let kind = err.kind();
    let msg = match kind {
        std::io::ErrorKind::NotFound => format!("{cmd:?}: not found"),
        _ => format!("{cmd:?}: {kind}"),
    };
    Error::new(err).context(msg)
}

fn run_ext(cmd: &mut Command, print: bool) -> Result<()> {
    let pp = pretty_print(cmd);
    log::info!("Running {pp}");
    if print {
        println!("{pp}");
    }
    let status = cmd
        .status()
        .map_err(|err| pretty_err(err, cmd))
        .context(pp.clone())?;
    if !status.success() {
        return Err(anyhow!(pp));
    }
    Ok(())
}

pub fn run_verbose(cmd: &mut Command) -> Result<()> {
    run_ext(cmd, true)
}

pub fn run(cmd: &mut Command) -> Result<()> {
    run_ext(cmd, false)
}

pub fn output(cmd: &mut Command) -> Result<String> {
    let pp = pretty_print(cmd);
    log::info!("Running {pp}");
    let output = cmd
        .output()
        .map_err(|err| pretty_err(err, cmd))
        .context(pp.clone())?;
    let err = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(anyhow!("{pp}: {err}"));
    }
    let stdout = output.stdout;
    let out = String::from_utf8(stdout.clone())
        .with_context(|| format!("failed to parse {pp} output {stdout:?} as utf8"))?;
    Ok(out)
}

pub fn output_xs(sh: &Shell, cmd: &Cmd) -> Result<String> {
    let pp = pretty_print_xs(sh, cmd);
    // FIXME: log::info!()
    let stdout = cmd.output()?.stdout;
    let out = String::from_utf8(stdout.clone())
        .with_context(|| format!("failed to parse {pp} output {stdout:?} as utf8"))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn shell() -> xshell::Shell {
        xshell::Shell::new().expect("Shell::new")
    }

    #[test]
    fn test_xs_to_string() {
        let sh = shell();
        let cmd = xshell::cmd!(sh, "hello world");

        assert_eq!(cmd.to_string(), "hello world");
    }

    #[test]
    fn test_pretty_print_xs() {
        let sh = shell();
        let cmd = xshell::cmd!(sh, "hello world");

        assert_eq!(pretty_print_xs(&sh, &cmd), "$ hello world");
    }

    #[test]
    fn test_pretty_print_xs_with_dir() {
        let sh = shell();
        sh.change_dir("/some/dir");
        let cmd = xshell::cmd!(sh, "hello world");

        assert_eq!(pretty_print_xs(&sh, &cmd), "/some/dir $ hello world");
    }

    #[cfg(unix)]
    #[test]
    fn test_output_xs() {
        let sh = shell();
        let cmd = xshell::cmd!(sh, "echo hello");

        assert_eq!(output_xs(&sh, &cmd).expect("output_xs").trim(), "hello");
    }
}
