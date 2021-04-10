use crate::args::Args;
use crate::errors::*;
use std::os::unix::ffi::OsStrExt;

pub fn unveil(args: &Args) -> Result<()> {
    if let Some(web_root) = &args.web_root {
        unveil::unveil(web_root, "r")
            .map_err(|e| anyhow!("Failed to unveil {:?}: {:?}", web_root, e))?;
    }
    if let Some(data_dir) = &args.data_dir {
        unveil::unveil(data_dir.as_os_str().as_bytes(), "rwc")
            .map_err(|e| anyhow!("Failed to unveil {:?}: {:?}", data_dir, e))?;
    }
    unveil::unveil("", "").map_err(|e| anyhow!("Failed to finish unveil: {:?}", e))?;
    Ok(())
}

pub fn pledge(args: &Args) -> Result<()> {
    let mut pledge = String::from("stdio dns inet rpath unix");
    if args.data_dir.is_some() {
        pledge.push_str(" wpath cpath id flock");
    }
    pledge::pledge(Some(pledge.as_str()), Some(""))?;
    Ok(())
}
