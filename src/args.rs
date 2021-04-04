use crate::errors::*;
use crate::utils;
use libtor::TorAddress;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
pub struct Args {
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
    /// The folder to store tor data in
    #[structopt(long)]
    pub data_dir: PathBuf,
    /// Only run the http server, without tor
    #[structopt(long)]
    pub skip_tor: bool,
    /// Files that should be served
    #[structopt(long)]
    pub web_root: String,
    /// Enable directory listing if no index.html was found
    #[structopt(short = "L", long)]
    pub list_directories: bool,
    /// The address to find to, supports unix domain sockets
    #[structopt(short = "B", long)]
    pub bind: Option<String>,
}

impl Args {
    pub fn hidden_service_path(&self) -> String {
        todo!()
    }

    pub fn data_path(&self) -> String {
        todo!()
    }

    pub fn socket_path(&self) -> String {
        todo!()
    }

    pub fn bind_addr(&self) -> Result<TorAddress> {
        if let Some(bind_addr) = &self.bind {
            let bind_addr = bind_addr.to_string();
            if bind_addr.starts_with('.') || bind_addr.starts_with('/') {
                Ok(TorAddress::Unix(bind_addr))
            } else {
                Ok(TorAddress::Address(bind_addr))
            }
        } else {
            let path = self.data_dir.join("narnia.sock");
            let path = utils::path_to_string(path)?;
            Ok(TorAddress::Unix(path))
        }
    }
}
