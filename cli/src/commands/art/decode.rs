use crate::commands::art::METADATA_FILE;
use crate::data::bank_meta::BankMetadata;
use anyhow::{ensure, Context};
use clap::Args;
use ct3lib::Art;
use std::env::current_dir;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct DecodeArgs {
    /// ART file to decode from
    files: Vec<PathBuf>,
    /// Directory to decode the contents into. Defaults to the current working directory.
    #[arg(short)]
    output_dir: Option<PathBuf>,
    /// Overwrite any existing files
    #[arg(short, long, default_value = "false")]
    force: bool,
}

impl DecodeArgs {
    pub fn handle(&self) -> anyhow::Result<()> {
        for file in &self.files {
            println!("Decoding {:?}...", file);
            self.handle_file(file)?;
        }

        Ok(())
    }

    fn handle_file(&self, file: &PathBuf) -> anyhow::Result<()> {
        ensure!(file.exists(), "File {:?} does not exist.", file);
        ensure!(!file.is_dir(), "Path {:?} cannot be a directory.", file);

        let bank_name = file.file_stem().context("Failed to retrieve file name")?;

        let output_dir = match &self.output_dir {
            Some(path) => path.to_owned(),
            None => current_dir()?,
        }
        .join(bank_name);

        if !self.force {
            ensure!(
                !output_dir.exists(),
                "Output directory {:?} already exists",
                output_dir
            );
        }

        create_dir_all(&output_dir)?;

        let decoder = Art::decode(BufReader::new(File::open(file)?))?;

        let mut headers = Vec::new();

        for (i, entry) in decoder.into_iter().enumerate() {
            let entry = entry?;
            headers.push(entry.header.clone());

            let png = entry.to_png();

            let file = File::create(output_dir.join(format!("{}.png", i)))?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&png)?;
        }

        let metadata =
            BankMetadata::from_headers(bank_name.to_os_string().into_string().unwrap(), headers);
        let file = File::create(output_dir.join(METADATA_FILE))?;
        let mut writer = BufWriter::new(file);
        let content = toml::to_string(&metadata)?;
        writer.write_all(content.as_ref())?;

        Ok(())
    }
}
