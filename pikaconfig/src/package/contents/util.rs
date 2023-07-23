use anyhow::{anyhow, Result};

use super::args::Arguments;

pub fn no_args(command: &str, args: &Arguments) -> Result<()> {
    if !args.0.is_empty() {
        return Err(anyhow!(
            "{} builder: want no arguments, got {}: {:?}",
            command,
            args.0.len(),
            args.0,
        ));
    }
    Ok(())
}

pub fn single_arg<'a>(command: &str, args: &'a Arguments) -> Result<&'a str> {
    Ok(&fixed_args(command, args, 1)?[0])
}

pub fn double_arg<'a>(command: &str, args: &'a Arguments) -> Result<(&'a str, &'a str)> {
    let args = fixed_args(command, args, 2)?;
    Ok((&args[0], &args[1]))
}

pub fn fixed_args<'a>(command: &str, args: &'a Arguments, len: usize) -> Result<&'a [String]> {
    if args.0.len() != len {
        return Err(anyhow!(
            "{command} builder: want {len} arguments, got {}: {args:?}",
            args.0.len(),
        ));
    }
    Ok(&args.0)
}

/// Returns (required_args, remainder_args).
pub fn multiple_args<'a>(
    command: &str,
    args: &'a Arguments,
    required: usize,
) -> Result<(&'a [String], &'a [String])> {
    if args.0.len() < required {
        return Err(anyhow!(
            "{} builder: want at least {} arguments, got {}: {:?}",
            command,
            required,
            args.0.len(),
            args.0,
        ));
    }
    let required_args = &args.0[0..required];
    let remainder_args = &args.0[required..];
    Ok((required_args, remainder_args))
}
