use crate::errors::*;
use libtor::TorAddress;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
    /// Enables a Tor thread for a hidden service, configures the folder to store Tor data in
    #[structopt(short = "D", long)]
    pub data_dir: Option<PathBuf>,
    /// Files that should be served
    #[structopt(short = "w", long)]
    pub web_root: Option<String>,
    /// Enable directory listing if no index.html was found
    #[structopt(short = "L", long)]
    pub list_directories: bool,
    /// The address to find to, supports unix domain sockets
    #[structopt(short = "B", long)]
    pub bind: Option<String>,
    #[cfg(unix)]
    /// Change the process to this user after setup
    #[structopt(short, long)]
    pub user: Option<String>,
    #[cfg(unix)]
    /// Chroot into folder before starting webserver
    #[structopt(short = "C", long)]
    pub chroot: Option<PathBuf>,
    /// Spawn a seperate process, read arguments as json from stdin
    #[structopt(short = "M", long)]
    pub child_process: bool,
    /// Always use multi-process mode
    #[structopt(short = "m", long)]
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
