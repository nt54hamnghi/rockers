use std::process::Command;

use nix::sched::CloneFlags;

use crate::cli::{RunArgs, TARGET};

impl RunArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        nix::sched::unshare(
            CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS,
        )?;
        nix::mount::mount(
            None::<&str>,
            "/",
            None::<&str>,
            nix::mount::MsFlags::MS_PRIVATE | nix::mount::MsFlags::MS_REC,
            None::<&str>,
        )?;
        nix::unistd::sethostname("container")?;

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
        nix::unistd::chroot(TARGET)?;
        nix::unistd::chdir("/")?;
        let mount_opts = MountOpts::new_mount_proc();
        mount_opts.mount()?;

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
    }
}

struct MountOpts<'m> {
    source: Option<&'m str>,
    target: &'m str,
    fstype: Option<&'m str>,
    flags: nix::mount::MsFlags,
    data: Option<&'m str>,
}

impl<'m> MountOpts<'m> {
    fn new_mount_proc() -> MountOpts<'m> {
        MountOpts {
            source: Some("proc"),
            target: "/proc",
            fstype: Some("proc"),
            flags: nix::mount::MsFlags::empty(),
            data: None::<&str>,
        }
    }

    fn mount(&self) -> anyhow::Result<()> {
        nix::mount::mount(self.source, self.target, self.fstype, self.flags, self.data)?;
        Ok(())
    }
}

impl<'m> Drop for MountOpts<'m> {
    fn drop(&mut self) {
        if let Some(source) = self.source {
            nix::mount::umount(source).expect("failed to umount proc");
        }
    }
}
