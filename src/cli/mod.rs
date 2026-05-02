use clap::{Args, Parser, Subcommand};

pub mod pull;
pub mod run;

#[derive(Debug, Parser)]
#[command(name = "rockers", about = "Pull and run container images")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Download an image from a registry
    Pull(PullArgs),
    /// Create and run a new container from an image
    Run(RunArgs),
    #[command(hide = true)]
    Child(RunArgs),
}

#[derive(Debug, Args, Clone)]
pub struct PullArgs {
    pub image: String,
}

#[derive(Debug, Args, Clone)]
pub struct RunArgs {
    // pub image: String,
    #[clap(required = true, trailing_var_arg = true)]
    pub command: Vec<String>,
}
