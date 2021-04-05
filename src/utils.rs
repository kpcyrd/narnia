use crate::errors::*;
use nix::sys::stat::Mode;
use std::path::{Path, PathBuf};

pub fn path_to_string(path: PathBuf) -> Result<String> {
    path.into_os_string()
        .into_string()
        .map_err(|e| anyhow!("Path contains invalid utf8: {:?}", e))
}

pub fn mkprivdir(path: &Path) -> Result<()> {
    match nix::unistd::mkdir(path, Mode::from_bits(0o700).unwrap()) {
        Ok(()) => Ok(()),
        Err(nix::Error::Sys(nix::errno::Errno::EEXIST)) => Ok(()),
        Err(err) => Err(Error::from(err)),
    }
}
