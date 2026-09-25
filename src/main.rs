use anyhow::Result;
use clap::Parser;
use cuv::commands::{self, Commands};

#[derive(Parser)]
#[command(name = "cuv")]
#[command(author = "Araskova <engineering@araskova.com>")]
#[command(version = "0.1.0")]
#[command(
    about = "Extremely fast, zero-dependency package manager and build driver for C/C++",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::dispatch(cli.command).await
}
