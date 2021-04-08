#![cfg(unix)]

use crate::errors::*;
use std::path::Path;

pub fn chroot(path: &Path) -> Result<()> {
    debug!("Attempting to chroot into {:?}", path);
    nix::unistd::chroot(path).context("Failed to chroot")?;
    nix::unistd::chdir("/").context("Failed to chdir after chroot")?;

    // we won't chroot twice, so drop all capabilities here
    #[cfg(target_os = "linux")]
    drop_caps()?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn drop_caps() -> Result<()> {
    use caps::CapSet;
    debug!("Dropping all capabilities");
    caps::clear(None, CapSet::Effective).context("Failed to clear effective capability set")?;
    caps::clear(None, CapSet::Permitted).context("Failed to clear permitted capability set")?;
    Ok(())
}
