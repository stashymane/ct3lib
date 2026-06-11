pub mod data;
pub mod png;
pub mod util;

pub use data::{ArtDecoder, ArtEncoder};
use crate::data::{Image, ImageHeader};
use std::io::{self, Read, Write};
use thiserror::Error;

const MAGIC: u32 = u32::from_le_bytes(*b"GXTX");

/// Convenience in-memory representation of an ART file.
#[derive(Debug, Clone)]
pub struct Art {
    pub images: Vec<Image>,
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("invalid magic at offset {offset}: got {got}")]
    InvalidMagic { offset: usize, got: u32 },
    #[error("unknown compression type: {value}")]
    UnknownCompression { value: u32 },
}

impl Art {
    pub fn decode<R: Read>(reader: R) -> Result<Self, DecodeError> {
        let mut decoder = ArtDecoder::new(reader)?;
        let mut images = Vec::with_capacity(decoder.len());
        while let Some((header, mut data_reader)) = decoder.next_entry()? {
            let mut data = Vec::new();
            data_reader.read_to_end(&mut data)?;
            images.push(Image { header, data });
        }
        Ok(Art { images })
    }

    pub fn encode<W: Write>(&self, writer: W) -> io::Result<()> {
        let entries: Vec<(ImageHeader, usize)> = self
            .images
            .iter()
            .map(|img| (img.header.clone(), img.data.len()))
            .collect();
        let mut encoder = ArtEncoder::new(writer, entries)?;
        for img in &self.images {
            encoder.write_image(img.data.as_slice())?;
        }
        Ok(())
    }
}

fn read_u16<R: Read>(reader: &mut R) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}
