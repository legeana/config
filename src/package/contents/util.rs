use super::builder;

use anyhow::{anyhow, Result};

/// Checks that the first argument is command and returns a slice of command arguments.
pub fn check_command<'a, 'b>(command: &str, args: &'a [&'b str]) -> Result<&'a [&'b str]> {
    if args.is_empty() {
        return Err(anyhow!("{} parser: got empty list", command));
    }
    let cmd = args[0];
    let cmd_args = &args[1..];
    if command != cmd {
        return Err(builder::Error::UnsupportedCommand {
            parser: command.to_owned(),
            command: cmd.to_owned(),
        }
        .into());
    }
    Ok(cmd_args)
}

pub fn no_args(command: &str, args: &[&str]) -> Result<()> {
    let cmd_args = check_command(command, args)?;
    if !cmd_args.is_empty() {
        return Err(anyhow!(
            "{} parser: want no arguments, got {}: {:?}",
            command,
            cmd_args.len(),
            cmd_args,
        ));
    }
    Ok(())
}

pub fn single_arg<'a>(command: &str, args: &[&'a str]) -> Result<&'a str> {
    Ok(fixed_args(command, args, 1)?[0])
}

pub fn fixed_args<'a, 'b>(command: &str, args: &'a [&'b str], len: usize) -> Result<&'a [&'b str]> {
    let cmd_args = check_command(command, args)?;
    if cmd_args.len() != len {
        return Err(anyhow!(
            "{command} parser: want {len} arguments, got {}: {cmd_args:?}",
            cmd_args.len(),
        ));
    }
    Ok(cmd_args)
}

/// Returns (required_args, remainder_args).
pub fn multiple_args<'a, 'b>(
    command: &str,
    args: &'a [&'b str],
    required: usize,
) -> Result<(&'a [&'b str], &'a [&'b str])> {
    let cmd_args = check_command(command, args)?;
    if cmd_args.len() < required {
        return Err(anyhow!(
            "{} parser: want at least {} arguments, got {}: {:?}",
            command,
            required,
            cmd_args.len(),
            cmd_args,
        ));
    }
    let required_args = &cmd_args[0..required];
    let remainder_args = &cmd_args[required..];
    Ok((required_args, remainder_args))
}
