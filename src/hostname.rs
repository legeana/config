use sysinfo::{System, SystemExt};

use anyhow::{anyhow, Result};

pub fn hostname() -> Result<String> {
    let sys = System::new();
    return sys.host_name().ok_or(anyhow!("failed to obtain hostname"));
}
