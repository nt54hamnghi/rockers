use std::path::Path;
use std::process::Command;

use anyhow::Context;
use nix::fcntl::AT_FDCWD;
use nix::mount::{MntFlags, MsFlags};
use nix::sched::CloneFlags;
use nix::sys::stat::Mode;
use nix::unistd::UnlinkatFlags;

use crate::cli::{RunArgs, TARGET};

impl RunArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        nix::sched::unshare(
            CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS,
        )
        .context("failed to create UTS, PID, and mount namespaces")?;
        // Set the mount propagation for the root mount point "/" to private recursively.
        // After this call, future mounts or unmounts inside this mount namespace will
        // not propagate to outside, and external mount changes will not propagate in.
        // This is done to prevent accidental propagation back to the host from the
        // container's mount namespace.
        nix::mount::mount(
            None::<&str>,
            "/",
            None::<&str>,
            nix::mount::MsFlags::MS_PRIVATE | nix::mount::MsFlags::MS_REC,
            None::<&str>,
        )
        .context("failed to make root mount private recursively")?;
        nix::unistd::sethostname("container").context("failed to set container hostname")?;

        let bin = "/proc/self/exe";
        let mut args = vec!["child"];
        args.extend(self.command.iter().map(String::as_str));

        let mut child = Command::new(bin).args(args).spawn()?;
        let status = child.wait().expect("command wasn't running");
        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            anyhow::bail!("process exited with code: {}", exit_code);
        }

        Ok(())
    }

    pub fn child(&self) -> anyhow::Result<()> {
        // nix::unistd::chroot(TARGET)?;

        // 1. bind mount rootfs, pivot_root requires the new root to be a mount point
        nix::mount::mount(
            Some(TARGET),
            TARGET,
            None::<&str>,
            MsFlags::MS_BIND,
            None::<&str>,
        )
        .with_context(|| format!("failed to bind mount rootfs at {TARGET}"))?;

        // 2. make .put_old/ under new root to host the old root
        let new_root = Path::new(TARGET);
        let put_old = new_root.join(".put_old");
        nix::unistd::mkdir(&put_old, Mode::from_bits_truncate(0o777)).with_context(|| {
            format!(
                "failed to create put_old directory at {}",
                put_old.display()
            )
        })?;

        // 3. perform pivot_root
        nix::unistd::chdir(TARGET)
            .with_context(|| format!("failed to change directory to new root at {TARGET}"))?;
        nix::unistd::pivot_root(".", ".put_old")
            .context("failed to pivot root with old root at .put_old")?;
        nix::unistd::chdir("/").context("failed to change directory to new root")?;

        // 4. lazily unmount old root and remove /.put_old
        nix::mount::umount2("/.put_old", MntFlags::MNT_DETACH)
            .context("failed to lazily unmount old root at /.put_old")?;
        nix::unistd::unlinkat(AT_FDCWD, "/.put_old", UnlinkatFlags::RemoveDir)
            .context("failed to remove old root directory at /.put_old")?;

        with_proc_mount(|| {
            let (bin, args) = self
                .command
                .split_first()
                .expect("run command is required by clap");

            let mut child = Command::new(bin).args(args).spawn()?;
            let status = child.wait().expect("command wasn't running");
            if !status.success() {
                let exit_code = status.code().unwrap_or(-1);
                anyhow::bail!("process exited with code: {}", exit_code);
            }
            Ok(())
        })?;

        Ok(())
    }
}

pub fn with_proc_mount(f: impl FnOnce() -> anyhow::Result<()>) -> anyhow::Result<()> {
    nix::mount::mount(
        Some("proc"),
        "/proc",
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )
    .context("failed to mount proc filesystem at /proc")?;
    let ret = f();
    nix::mount::umount("proc").context("failed to unmount proc filesystem")?;
    ret
}
