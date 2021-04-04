use crate::errors::*;
use std::path::PathBuf;

pub fn path_to_string(path: PathBuf) -> Result<String> {
    path.into_os_string()
        .into_string()
        .map_err(|e| anyhow!("Path contains invalid utf8: {:?}", e))
}
