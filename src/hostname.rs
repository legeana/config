use sysinfo::{System, SystemExt};

use anyhow::{anyhow, Result};

pub fn hostname() -> Result<String> {
    let sys = System::new();
    sys.host_name()
        .ok_or_else(|| anyhow!("failed to obtain hostname"))
}
