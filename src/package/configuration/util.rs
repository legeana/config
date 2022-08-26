use crate::package::configuration::parser::{Error, Result};

use anyhow::anyhow;

fn check_command(command: &str, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        return Err(Error::Other(anyhow!("{} parser: got empty list", command)));
    }
    if command != args[0] {
        return Err(Error::UnsupportedCommand {
            parser: command.to_string(),
            command: args[0].to_string(),
        });
    }
    return Ok(());
}

pub fn single_arg<'a>(command: &str, args: &[&'a str]) -> Result<&'a str> {
    check_command(command, args)?;
    if args.len() != 2 {
        return Err(Error::Other(anyhow!(
            "{} parser: want a single argument, got {}: {:?}",
            command,
            args.len(),
            args
        )));
    }
    return Ok(args[1]);
}
