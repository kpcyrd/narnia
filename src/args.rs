use crate::errors::*;
use crate::utils;
use libtor::TorAddress;
use std::path::PathBuf;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
    /// Enables a Tor thread for a hidden service, configures the folder to store Tor data in
    #[structopt(short = "D", long)]
    pub data_dir: Option<PathBuf>,
    /// Files that should be served
    #[structopt(short = "w", long)]
    pub web_root: String,
    /// Enable directory listing if no index.html was found
    #[structopt(short = "L", long)]
    pub list_directories: bool,
    /// The address to find to, supports unix domain sockets
    #[structopt(short = "B", long)]
    pub bind: Option<String>,
    /// Chroot into folder before starting webserver
    #[structopt(short = "C", long)]
    pub chroot: Option<PathBuf>,
}

impl Args {
    pub fn bind_addr(&self) -> Result<TorAddress> {
        if let Some(bind_addr) = &self.bind {
            let bind_addr = bind_addr.to_string();
            if bind_addr.starts_with('.') || bind_addr.starts_with('/') {
                Ok(TorAddress::Unix(bind_addr))
            } else {
                Ok(TorAddress::Address(bind_addr))
            }
        } else if let Some(data_dir) = &self.data_dir {
            let path = data_dir.join("narnia.sock");
            let path = utils::path_to_string(path)?;
            Ok(TorAddress::Unix(path))
        } else {
            bail!("Either bind address or data directory needs to be configured")
        }
    }
}
