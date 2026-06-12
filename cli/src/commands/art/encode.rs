use crate::commands::art::{resolve_file_or_cwd, METADATA_FILE};
use crate::data::bank_meta::BankMetadata;
use anyhow::ensure;
use clap::Args;
use ct3lib::data::Image;
use ct3lib::Art;
use std::fs::{read, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

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

impl EncodeArgs {
    pub fn handle(&self) -> anyhow::Result<()> {
        let directory = &self.directory;

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

        let metadata_path = directory.join(METADATA_FILE);
        ensure!(
            metadata_path.exists(),
            "Metadata file does not exist at {:?}",
            metadata_path
        );
        let metadata: BankMetadata = toml::from_slice(&read(metadata_path)?)?;

        let output_path = resolve_file_or_cwd(&self.output, || metadata.get_filename())?;
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

        let file = File::create(output_path)?;
        Art::encode(BufWriter::new(file), &images)?;

        Ok(())
    }
}
