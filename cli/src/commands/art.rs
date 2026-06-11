use crate::data::bank_meta::BankMetadata;
use anyhow::{ensure, Context};
use clap::{Args, Subcommand};
use ct3lib::data::Image;
use ct3lib::Art;
use std::env::current_dir;
use std::fs::{create_dir_all, read, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Args)]
pub struct ArtArgs {
    #[command(subcommand)]
    command: ArtCommands,
}

#[derive(Debug, Subcommand)]
pub enum ArtCommands {
    /// Encode an ART image
    Encode(EncodeArgs),
    /// Decode an ART image
    Decode(DecodeArgs),
}

#[derive(Debug, Args)]
pub struct EncodeArgs {
    /// Directory to encode the file from
    directory: PathBuf,
    /// ART file path to output. Defaults to the current working directory.
    #[arg(short)]
    output: Option<PathBuf>,
    /// Overwrite existing files
    #[arg(short, long, default_value = "false")]
    force: bool,
}

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// ART file to decode from
    file: PathBuf,
    /// Directory to decode the contents into. Defaults to the current working directory.
    #[arg(short)]
    output_dir: Option<PathBuf>,
    /// Overwrite any existing files
    #[arg(short, long, default_value = "false")]
    force: bool,
}

const METADATA_JSON: &str = "metadata.json";

impl ArtArgs {
    pub fn handle(&self) -> anyhow::Result<()> {
        match &self.command {
            ArtCommands::Encode(args) => {
                let directory = &args.directory;
                ensure!(
                    directory.exists(),
                    "Directory {:?} does not exist.",
                    directory
                );
                ensure!(
                    directory.is_dir(),
                    "Path {:?} is not a directory.",
                    directory
                );

                let metadata_path = directory.join(METADATA_JSON);
                ensure!(
                    metadata_path.exists(),
                    "Metadata file does not exist at {:?}",
                    metadata_path
                );
                let metadata: BankMetadata = serde_json::from_slice(&read(metadata_path)?)?;

                let output_path = resolve_file_or_cwd(&args.output, || metadata.get_filename())?;
                ensure!(
                    output_path.parent().is_some(),
                    "Output path cannot be empty"
                );

                let mut images: Vec<Image> = Vec::new();

                for (i, img_meta) in &metadata.metadata {
                    let png_path = directory.join(format!("{}.png", i));
                    ensure!(
                        png_path.exists(),
                        "PNG file does not exist at {:?}",
                        png_path
                    );
                    images.push(Image::from_png(
                        Path::new(&png_path),
                        img_meta.compression,
                        img_meta.mip_count,
                    )?);
                }

                let art = Art { images };
                let file = File::create(output_path)?;
                art.encode(BufWriter::new(file))?;

                Ok(())
            }

            ArtCommands::Decode(args) => {
                let file = &args.file;
                ensure!(file.exists(), "File {:?} does not exist.", file);
                ensure!(!file.is_dir(), "Path {:?} cannot be a directory.", file);

                let bank_name = file.file_stem().context("Failed to retrieve file name")?;

                let output_dir = match &args.output_dir {
                    Some(path) => path.to_owned(),
                    None => current_dir()?,
                }
                .join(bank_name);
                if !args.force {
                    ensure!(
                        !output_dir.exists(),
                        "Output directory {:?} already exists",
                        output_dir
                    );
                }

                create_dir_all(&output_dir)?;

                let art = Art::decode(BufReader::new(File::open(file)?))?;

                {
                    let metadata =
                        BankMetadata::from(bank_name.to_os_string().into_string().unwrap(), &art);

                    let file = File::create(output_dir.join(METADATA_JSON))?;
                    let writer = BufWriter::new(file);
                    serde_json::to_writer_pretty(writer, &metadata)?;
                }

                for (i, image) in art.images.iter().enumerate() {
                    let png = image.to_png();
                    let file = File::create(output_dir.join(format!("{}.png", i)))?;
                    let mut writer = BufWriter::new(file);
                    writer.write_all(&png)?;
                }

                Ok(())
            }
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
