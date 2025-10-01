use anyhow::{Result, bail};

pub fn init(quiet: bool, verbosity: u8) -> Result<()> {
    env_logger::Builder::new()
        .filter_level(match (quiet, verbosity) {
            (true, 0) => log::LevelFilter::Off,
            (true, _) => bail!("can't set quiet and verbose at the same time"),
            (false, 0) => log::LevelFilter::Error,
            (false, 1) => log::LevelFilter::Warn,
            (false, 2) => log::LevelFilter::Info,
            (false, 3) => log::LevelFilter::Debug,
            (false, 4) => log::LevelFilter::Trace,
            (false, 5..) => bail!("invalid log level: {verbosity}"),
        })
        .default_format()
        .format_timestamp(None)
        .format_target(false)
        .try_init()?;
    Ok(())
}
