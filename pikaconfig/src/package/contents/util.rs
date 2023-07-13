use anyhow::{anyhow, Result};

/// Checks that the first argument is command and returns a slice of command arguments.
pub fn check_command<'a, 'b>(command: &str, args: &'a [&'b str]) -> Result<&'a [&'b str]> {
    if args.is_empty() {
        return Err(anyhow!("{} builder: got empty list", command));
    }
    let cmd = args[0];
    let cmd_args = &args[1..];
    if command != cmd {
        return Err(anyhow!(
            "incorrect command: expected {command:?}, got {cmd:?}"
        ));
    }
    Ok(cmd_args)
}

pub fn no_args(command: &str, args: &[&str]) -> Result<()> {
    let cmd_args = check_command(command, args)?;
    if !cmd_args.is_empty() {
        return Err(anyhow!(
            "{} builder: want no arguments, got {}: {:?}",
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

pub fn double_arg<'a>(command: &str, args: &[&'a str]) -> Result<(&'a str, &'a str)> {
    let args = fixed_args(command, args, 2)?;
    Ok((args[0], args[1]))
}

pub fn fixed_args<'a, 'b>(command: &str, args: &'a [&'b str], len: usize) -> Result<&'a [&'b str]> {
    let cmd_args = check_command(command, args)?;
    if cmd_args.len() != len {
        return Err(anyhow!(
            "{command} builder: want {len} arguments, got {}: {cmd_args:?}",
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
            "{} builder: want at least {} arguments, got {}: {:?}",
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
