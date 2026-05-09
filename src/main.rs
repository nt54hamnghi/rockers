use clap::Parser;
use rockers::cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Pull(p) => p.run().await,
        Command::Run(r) => r.run(),
        Command::Child(r) => r.child(),
    }
}
