use anyhow::{anyhow, Result};

pub fn no_args(command: &str, args: &[&str]) -> Result<()> {
    if !args.is_empty() {
        return Err(anyhow!(
            "{} builder: want no arguments, got {}: {:?}",
            command,
            args.len(),
            args,
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
    if args.len() != len {
        return Err(anyhow!(
            "{command} builder: want {len} arguments, got {}: {args:?}",
            args.len(),
        ));
    }
    Ok(args)
}

/// Returns (required_args, remainder_args).
pub fn multiple_args<'a, 'b>(
    command: &str,
    args: &'a [&'b str],
    required: usize,
) -> Result<(&'a [&'b str], &'a [&'b str])> {
    if args.len() < required {
        return Err(anyhow!(
            "{} builder: want at least {} arguments, got {}: {:?}",
            command,
            required,
            args.len(),
            args,
        ));
    }
    let required_args = &args[0..required];
    let remainder_args = &args[required..];
    Ok((required_args, remainder_args))
}
