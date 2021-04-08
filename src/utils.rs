use crate::errors::*;
use std::path::PathBuf;
#[cfg(unix)]
use std::path::Path;

pub fn path_to_string(path: PathBuf) -> Result<String> {
    path.into_os_string()
        .into_string()
        .map_err(|e| anyhow!("Path contains invalid utf8: {:?}", e))
}

#[cfg(unix)]
pub fn mkprivdir(path: &Path) -> Result<()> {
    use nix::sys::stat::Mode;
    match nix::unistd::mkdir(path, Mode::from_bits(0o700).unwrap()) {
        Ok(()) => Ok(()),
        Err(nix::Error::Sys(nix::errno::Errno::EEXIST)) => Ok(()),
        Err(err) => Err(Error::from(err)),
    }
}
