use env_logger::Env;
use narnia::args::Args;
use narnia::errors::*;
use nix::sys::stat::Mode;
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;

fn main() -> Result<()> {
    let args = Args::from_args();
    let log_level = match args.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    env_logger::init_from_env(Env::default().default_filter_or(log_level));

    match nix::unistd::mkdir(&args.data_dir, Mode::from_bits(0o700).unwrap()) {
        Ok(()) => Ok(()),
        Err(nix::Error::Sys(nix::errno::Errno::EEXIST)) => Ok(()),
        Err(err) => Err(err)
            .with_context(|| anyhow!("Failed to create data directory: {:?}", &args.data_dir)),
    }?;

    let (tx, rx) = mpsc::channel();
    {
        let tx = tx.clone();
        let args = args.clone();
        thread::spawn(move || {
            if let Err(err) = narnia::httpd::run(args) {
                error!("httpd thread has terminated: {:#}", err);
            }
            tx.send(()).ok();
        });
    }
    if !args.skip_tor {
        thread::spawn(move || {
            if let Err(err) = narnia::tor::run(args) {
                error!("Tor thread has terminated: {:#}", err);
            }
            tx.send(()).ok();
        });
    }

    let _ = rx.recv();
    Ok(())
}
