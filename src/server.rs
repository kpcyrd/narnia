use crate::args::Args;
use crate::errors::*;
use crate::httpd;
#[cfg(unix)]
use crate::utils;
use libtor::TorAddress;
use std::env;
use std::io::Write;
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::net::UnixListener;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub enum ServerType {
    Thread((Bind, String)),
    Child((Child, ChildStdin)),
}

pub struct Server {
    inner: ServerType,
    args: Args,
    tx: mpsc::Sender<()>,
}

impl Server {
    pub fn setup(mut args: Args, tx: mpsc::Sender<()>) -> Result<Server> {
        let inner = if args.needs_child() {
            debug!("Setting up httpd child process");
            args.child_process = false;
            args.always_multi_process = false;
            args.data_dir = None;
            let json = serde_json::to_string(&args)?;

            debug!("Spawning multi-process child");
            let exe = env::current_exe().context("Failed to get own path")?;
            let mut cmd = Command::new(exe)
                .args(&["-M"])
                .stdin(Stdio::piped())
                .spawn()
                .context("Failed to spawn child")?;

            let mut stdin = cmd.stdin.take().unwrap();
            stdin.write_all(json.as_bytes())?;
            stdin.write_all(b"\n")?;
            debug!("Sent instructions to child");

            ServerType::Child((cmd, stdin))
        } else {
            debug!("Setting up httpd");

            let bind = Bind::setup(&args).context("Failed to bind socket")?;
            let web_root = args
                .web_root
                .clone()
                .context("Missing --web-root argument")?;

            ServerType::Thread((bind, web_root))
        };
        Ok(Server { inner, args, tx })
    }

    pub fn background(self) {
        thread::spawn(move || {
            match self.inner {
                ServerType::Child((mut cmd, _)) => {
                    let status = cmd.wait();
                    error!("child process has exited: {:?}", status);
                }
                ServerType::Thread((bind, web_root)) => {
                    if let Err(err) = httpd::run(self.args, bind, web_root) {
                        error!("httpd thread has terminated: {:#}", err);
                    }
                }
            }
            self.tx.send(()).ok();
        });
    }
}

pub enum Bind {
    Tcp(TcpListener),
    #[cfg(unix)]
    Unix(UnixListener),
}

impl Bind {
    pub fn setup(args: &Args) -> Result<Bind> {
        #[cfg(unix)]
        if let Some(data_dir) = &args.data_dir {
            utils::mkprivdir(&data_dir)
                .with_context(|| anyhow!("Failed to create data directory: {:?}", &data_dir))?;
        }

        let bind = match args.bind_addr()? {
            TorAddress::Address(addr) => {
                info!("Binding to tcp: {:?}", addr);
                let listener = TcpListener::bind(addr)?;
                Bind::Tcp(listener)
            }
            #[cfg(unix)]
            TorAddress::Unix(path) => {
                info!("Binding to unix: {:?}", path);
                let listener = UnixListener::bind(path)?;
                Bind::Unix(listener)
            }
            _ => unreachable!(),
        };
        Ok(bind)
    }
}
