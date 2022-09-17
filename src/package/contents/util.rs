use super::parser;

use anyhow::anyhow;

/// Checks that the first argument is command and returns a slice of command arguments.
pub fn check_command<'a, 'b>(command: &str, args: &'a [&'b str]) -> parser::Result<&'a [&'b str]> {
    if args.is_empty() {
        return Err(anyhow!("{} parser: got empty list", command).into());
    }
    let cmd = args[0];
    let cmd_args = &args[1..];
    if command != cmd {
        return Err(parser::Error::UnsupportedCommand {
            parser: command.to_owned(),
            command: cmd.to_owned(),
        });
    }
    Ok(cmd_args)
}

pub fn no_args(command: &str, args: &[&str]) -> parser::Result<()> {
    let cmd_args = check_command(command, args)?;
    if !cmd_args.is_empty() {
        return Err(anyhow!(
            "{} parser: want no arguments, got {}: {:?}",
            command,
            cmd_args.len(),
            cmd_args,
        )
        .into());
    }
    Ok(())
}

pub fn single_arg<'a>(command: &str, args: &[&'a str]) -> parser::Result<&'a str> {
    let cmd_args = check_command(command, args)?;
    if cmd_args.len() != 1 {
        return Err(anyhow!(
            "{} parser: want a single argument, got {}: {:?}",
            command,
            cmd_args.len(),
            cmd_args,
        )
        .into());
    }
    Ok(args[1])
}

/// Returns (required_args, remainder_args).
pub fn multiple_args<'a, 'b>(
    command: &str,
    args: &'a [&'b str],
    required: usize,
) -> parser::Result<(&'a [&'b str], &'a [&'b str])> {
    let cmd_args = check_command(command, args)?;
    if cmd_args.len() < required {
        return Err(anyhow!(
            "{} parser: want at least {} arguments, got {}: {:?}",
            command,
            required,
            cmd_args.len(),
            cmd_args,
        )
        .into());
    }
    let required_args = &cmd_args[0..required];
    let remainder_args = &cmd_args[required..];
    Ok((required_args, remainder_args))
}
