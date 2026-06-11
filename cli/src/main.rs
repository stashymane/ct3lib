use crate::commands::root::{Cli, Commands};
use clap::Parser;

pub mod commands;
pub mod data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();
    match cli.command {
        Commands::Art(args) => args.handle()?,
    }

    Ok(())
}
