use crate::commands::art::{resolve_file_or_cwd, METADATA_FILE};
use crate::data::bank_meta::BankMetadata;
use anyhow::ensure;
use clap::Args;
use ct3lib::art::image::Image;
use ct3lib::art::Art;
use log::warn;
use std::fs::{read, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

#[derive(Debug, Args)]
pub struct EncodeArgs {
    /// Directories to encode into ART files
    directories: Vec<PathBuf>,
    /// ART file path to output
    #[arg(short)]
    output: Option<PathBuf>,
    /// Overwrite existing files
    #[arg(short, long, default_value = "false")]
    force: bool,
}

impl EncodeArgs {
    pub fn handle(&self) -> anyhow::Result<()> {
        for dir in &self.directories {
            println!("Encoding {:?}...", dir);
            self.handle_directory(dir)?;
        }

        println!("Done!");

        Ok(())
    }

    pub fn handle_directory(&self, directory: &PathBuf) -> anyhow::Result<()> {
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
            let dds_path = directory.join(format!("{}.dds", i));
            ensure!(
                dds_path.exists(),
                "DDS file does not exist at {:?}",
                dds_path
            );

            let dds_bytes = read(&dds_path)?;
            if let Some(dds_compression) = Image::compression_from_dds_bytes(&dds_bytes) {
                if dds_compression != img_meta.compression {
                    warn!(
                        "Image {i}: DDS pixel format ({:?}) does not match metadata compression ({:?}). \
                         Re-encoding as {:?}.",
                        dds_compression, img_meta.compression, img_meta.compression
                    );
                }
            }

            images.push(Image::from_dds(
                Path::new(&dds_path),
                img_meta.compression,
                img_meta.mip_count,
            )?);
        }

        let file = File::create(output_path)?;
        Art::encode(BufWriter::new(file), &images)?;

        Ok(())
    }
}
