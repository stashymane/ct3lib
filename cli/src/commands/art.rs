pub mod decode;
pub mod encode;
pub mod info;

use crate::commands::art::decode::DecodeArgs;
use crate::commands::art::encode::EncodeArgs;
use crate::commands::art::info::InfoArgs;
use anyhow::Context;
use clap::{Args, Subcommand};
use std::env::current_dir;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ArtArgs {
    #[command(subcommand)]
    command: ArtCommands,
}

#[derive(Debug, Subcommand)]
pub enum ArtCommands {
    /// View information about an ART image bank
    Info(InfoArgs),
    /// Encode an ART image bank
    Encode(EncodeArgs),
    /// Decode an ART image bank
    Decode(DecodeArgs),
}

const METADATA_FILE: &str = "metadata.toml";

impl ArtArgs {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            ArtCommands::Info(args) => args.handle().context("Failed to run info command"),
            ArtCommands::Encode(args) => args.handle().context("Failed to run encode command"),
            ArtCommands::Decode(args) => args.handle().context("Failed to run decode command"),
        }
    }
}

fn resolve_file_or_cwd<F>(path: &Option<PathBuf>, default_filename: F) -> std::io::Result<PathBuf>
where
    F: Fn() -> String,
{
    Ok(match path {
        Some(output) => {
            if output.extension().is_some() {
                output.to_path_buf()
            } else {
                output.join(default_filename())
            }
        }
        None => current_dir()?.join(default_filename()),
    })
}
