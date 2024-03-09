use anyhow::{bail, Result};

pub fn init(verbosity: u8) -> Result<()> {
    env_logger::Builder::new()
        .filter_level(match verbosity {
            0 => log::LevelFilter::Off,
            1 => log::LevelFilter::Error,
            2 => log::LevelFilter::Warn,
            3 => log::LevelFilter::Info,
            4 => log::LevelFilter::Debug,
            5 => log::LevelFilter::Trace,
            6.. => bail!("invalid log level: {}", verbosity),
        })
        .default_format()
        .format_timestamp(None)
        .format_target(false)
        .try_init()?;
    Ok(())
}
