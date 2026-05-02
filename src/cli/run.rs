use std::os::unix::process::CommandExt;
use std::process::Command;

use nix::sched::CloneFlags;

use crate::cli::RunArgs;

impl RunArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let (bin, args) = self
            .command
            .split_first()
            .expect("run command is required by clap");

        println!("Running: {:?}, with: {:?}", bin, args);

        let mut cmd = Command::new(bin);
        cmd.args(args);

        // SAFETY: `pre_exec` runs in the child after `fork` and before `exec`
        // This closure only performs `unshare` and does not use allocate memory
        // access environment variables with std::env, or acquire a mutex.
        unsafe {
            cmd.pre_exec(|| {
                nix::sched::unshare(CloneFlags::CLONE_NEWUTS)?;
                Ok(())
            });
        }

        let mut child = cmd.spawn()?;
        let status = child.wait().expect("command wasn't running");

        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            anyhow::bail!("process exited with code: {}", exit_code);
        }

        Ok(())
    }
}
