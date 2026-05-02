use clap::Parser;
use rockers::cli::{Cli, Command};
use tokio::runtime::Builder;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Pull(pull) => Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(pull.run()),
        Command::Run(run) => run.run(),
        Command::Child(run) => run.child(),
    }
}
