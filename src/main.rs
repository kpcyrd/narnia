use env_logger::Env;
use narnia::args::Args;
use narnia::errors::*;
use narnia::security;
use narnia::server::Server;
use std::io::{self, Read};
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;

fn get_arguments() -> Result<Args> {
    let mut args = Args::from_args();
    if args.child_process {
        args = read_args_stdin().context("Failed to read arguments from stdin")?;
        args.child_process = true;
    }
    Ok(args)
}

fn read_args_stdin() -> Result<Args> {
    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.read_line(&mut buf)?;
    debug!("Received arguments from parent: {:?}", buf);
    let args = serde_json::from_str(&buf).context("Failed to parse json")?;
    Ok(args)
}

fn main() -> Result<()> {
    let args = get_arguments()?;

    let log_level = match args.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    env_logger::init_from_env(Env::default().default_filter_or(log_level));

    let (tx, rx) = mpsc::channel();

    let server = Server::setup(args.clone(), tx.clone())?;
    debug!("Locking down process");
    security::setup(&args)?;
    debug!("Sending server to background");
    server.background();

    // if we are a child process we monitor if stdin gets closed so we shutdown if the parent dies
    if args.child_process {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buf = [0; 128];
            loop {
                match stdin.read(&mut buf) {
                    Ok(n) if n == 0 => {
                        warn!("Detected stdin was closed, shutting down");
                        break;
                    }
                    Ok(_) => (),
                    Err(err) => {
                        warn!("Failed reading from stdin, shutting down: {:#}", err);
                        break;
                    }
                }
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
