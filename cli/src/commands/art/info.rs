use anyhow::{ensure, Context};
use clap::Args;
use ct3lib::art::image_header::ImageHeader;
use ct3lib::art::Art;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct InfoArgs {
    /// Path to an ART file
    path: PathBuf,
    /// ID of the image to inspect
    image: Option<usize>,
    /// List all images
    #[arg(long, default_value = "false")]
    all: bool,
}

impl InfoArgs {
    pub fn handle(&self) -> anyhow::Result<()> {
        let path = &self.path;
        ensure!(path.exists(), "Provided path {:?} does not exist", path);
        ensure!(path.is_file(), "Provided path {:?} is not a file", path);

        let reader = BufReader::new(File::open(path).context("Failed to open file")?);
        let mut decoder = Art::decode(reader).context("Failed to decode ART file")?;

        if self.all {
            for (i, entry) in decoder.into_iter().enumerate() {
                let entry = entry?;
                print!("[{}] ", i);
                print_header(&entry.header);
            }
        } else if let Some(id) = self.image {
            let header = decoder.header_at(id)?;
            print_header(&header);
        } else {
            println!(
                "{}",
                path.file_name()
                    .expect("File name should exist")
                    .to_string_lossy()
            );
            println!("Total images: {}", decoder.len());
            //TODO have a lookup of what this bank is used for based on decompilation
        }

        Ok(())
    }
}

fn print_header(header: &ImageHeader) {
    println!(
        "{}x{} \tcompression={:?} \tmip_count={} \tsize={}",
        header.width, header.height, header.compression, header.mip_count, header.size
    );
}
