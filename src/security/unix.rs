use crate::args::Args;
use crate::errors::*;
use nix::unistd::{Gid, Uid};
use std::path::Path;
use users::User;

pub fn setup(args: &Args) -> Result<()> {
    let user = if let Some(user) = &args.user {
        let user = users::get_user_by_name(&user).context("Could not find user")?;
        Some(user)
    } else {
        None
    };

    if args.data_dir.is_none() {
        if let Some(path) = &args.chroot {
            chroot(&path).with_context(|| anyhow!("Failed to chroot into: {:?}", path))?;
            info!("Successfully chrooted into {:?}", path);
        }
    }

    if let Some(user) = user {
        become_user(&user)?;
    }

    // we won't chroot twice, so drop all capabilities here
    #[cfg(target_os = "linux")]
    drop_caps()?;

    Ok(())
}

fn chroot(path: &Path) -> Result<()> {
    debug!("Attempting to chroot into {:?}", path);
    nix::unistd::chroot(path).context("Failed to chroot")?;
    nix::unistd::chdir("/").context("Failed to chdir after chroot")?;
    Ok(())
}

fn become_user(user: &User) -> Result<()> {
    let uid = Uid::from_raw(user.uid());
    let gid = Gid::from_raw(user.primary_group_id());

    debug!("Changing user to uid={}, gid={}", uid, gid);
    #[cfg(target_os = "linux")]
    nix::unistd::setgroups(&[]).context("Failed to clear supplementary groups")?;
    nix::unistd::setgid(gid).context("Failed to set gid")?;
    nix::unistd::setuid(uid).context("Failed to set uid")?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn drop_caps() -> Result<()> {
    use caps::CapSet;
    debug!("Dropping all capabilities");
    caps::clear(None, CapSet::Effective).context("Failed to clear effective capability set")?;
    caps::clear(None, CapSet::Permitted).context("Failed to clear permitted capability set")?;
    Ok(())
}
