use env_logger::Env;
use narnia::args::Args;
use narnia::errors::*;
use narnia::utils;
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

    if let Some(data_dir) = &args.data_dir {
        utils::mkprivdir(&data_dir)
            .with_context(|| anyhow!("Failed to create data directory: {:?}", &data_dir))?;
    }

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
    if let Some(data_dir) = args.data_dir.clone() {
        thread::spawn(move || {
            if let Err(err) = narnia::tor::run(args, data_dir) {
                error!("Tor thread has terminated: {:#}", err);
            }
            tx.send(()).ok();
        });
    }

    let _ = rx.recv();
    Ok(())
}
