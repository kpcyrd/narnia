use crate::errors::*;
use libtor::TorAddress;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, clap::Parser, Serialize, Deserialize)]
pub struct Args {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
    /// Enables a Tor thread for a hidden service, configures the folder to store Tor data in
    #[clap(short = 'D', long, env = "NARNIA_DATA_DIR")]
    pub data_dir: Option<PathBuf>,
    /// Files that should be served
    #[clap(short = 'w', long, env = "NARNIA_WEB_ROOT")]
    pub web_root: Option<String>,
    /// Enable directory listing if no index.html was found
    #[clap(short = 'L', long)]
    pub list_directories: bool,
    /// The address to find to, supports unix domain sockets
    #[clap(short = 'B', long, env = "NARNIA_BIND_ADDR")]
    pub bind: Option<String>,
    #[cfg(unix)]
    /// Change the process to this user after setup
    #[clap(short, long)]
    pub user: Option<String>,
    #[cfg(unix)]
    /// Chroot into folder before starting webserver
    #[clap(short = 'C', long)]
    pub chroot: Option<PathBuf>,
    /// Spawn a seperate process, read arguments as json from stdin
    #[clap(short = 'M', long)]
    pub child_process: bool,
    /// Always use multi-process mode
    #[clap(short = 'm', long)]
    pub always_multi_process: bool,
}

impl Args {
    pub fn needs_child(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                self.always_multi_process || (self.data_dir.is_some() && self.chroot.is_some())
            } else {
                self.always_multi_process
            }
        }
    }

    pub fn bind_addr(&self) -> Result<TorAddress> {
        if let Some(bind_addr) = &self.bind {
            let bind_addr = bind_addr.to_string();
            cfg_if::cfg_if! {
                if #[cfg(unix)] {
                    if bind_addr.starts_with('.') || bind_addr.starts_with('/') {
                        Ok(TorAddress::Unix(bind_addr))
                    } else {
                        Ok(TorAddress::Address(bind_addr))
                    }
                } else {
                    Ok(TorAddress::Address(bind_addr))
                }
            }
        } else if let Some(data_dir) = &self.data_dir {
            cfg_if::cfg_if! {
                if #[cfg(unix)] {
                    use crate::utils;
                    let path = data_dir.join("narnia.sock");
                    let path = utils::path_to_string(path)?;
                    Ok(TorAddress::Unix(path))
                } else {
                    let _ = data_dir;
                    bail!("You always have to set -B on windows");
                }
            }
        } else {
            bail!("Either bind address or data directory needs to be configured")
        }
    }
}
