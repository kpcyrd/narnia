#[cfg_attr(unix, path = "unix.rs")]
#[cfg_attr(windows, path = "windows.rs")]
mod os;

pub use self::os::*;

#[cfg(target_os = "openbsd")]
pub mod openbsd;
