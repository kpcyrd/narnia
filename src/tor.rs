use crate::args::Args;
use crate::errors::*;
use crate::utils;
use libtor::{HiddenServiceVersion, Tor, TorAddress, TorFlag};

pub fn run(args: Args) -> Result<()> {
    let bind_addr = args.bind_addr()?;

    let hs_path = utils::path_to_string(args.data_dir.join("hs"))?;
    let data_dir = utils::path_to_string(args.data_dir)?;

    Tor::new()
        .flag(TorFlag::DataDirectory(data_dir))
        .flag(TorFlag::SocksPort(0))
        .flag(TorFlag::HiddenServiceDir(hs_path))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::HiddenServicePort(
            TorAddress::Port(80),
            Some(bind_addr).into(),
        ))
        .start()
        .context("Failed to start tor")?;

    warn!("Tor thread has terminated");
    Ok(())
}
