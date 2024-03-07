use anyhow::{bail, Result};

use pikaconfig_bootstrap::cli;

fn main() -> Result<()> {
    let args = cli::parse();
    env_logger::Builder::new()
        .filter_level(match args.verbose {
            0 => log::LevelFilter::Off,
            1 => log::LevelFilter::Error,
            2 => log::LevelFilter::Warn,
            3 => log::LevelFilter::Info,
            4 => log::LevelFilter::Debug,
            5 => log::LevelFilter::Trace,
            6.. => bail!("invalid log level: {}", args.verbose),
        })
        .default_format()
        .format_timestamp(None)
        .format_target(false)
        .try_init()?;
    bail!("Not implemented yet");
}
