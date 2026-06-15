use crate::art::decoder::{ArtDecoder, DecodeResult};
use crate::art::encoder::ArtEncoder;
use crate::art::image::Image;
use crate::art::image_header::ImageHeader;
use std::io;
use std::io::{Read, Write};

pub mod compression;
pub mod dds;
pub mod decoder;
pub mod encoder;
pub mod image;
pub mod image_header;
pub mod png;

pub(crate) const MAGIC: u32 = u32::from_le_bytes(*b"GXTX");

/// Entrypoint for creating ART encoders and decoders.
///
/// Use [`Art::decode`] to open an ART stream for reading, or [`Art::encode`]
/// to write a collection of images into an ART stream.
pub struct Art;

impl Art {
    /// Open an ART stream for reading. The file header is read immediately;
    /// image data is read on demand via the returned [`ArtDecoder`].
    ///
    /// # Example
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// use ct3lib::art::Art;
    ///
    /// let decoder = Art::decode(BufReader::new(File::open("bank.art").unwrap())).unwrap();
    /// for entry in decoder {
    ///     let entry = entry.unwrap();
    ///     println!("{:?}", entry.header);
    ///     let rgba = entry.decode();
    /// }
    /// ```
    pub fn decode<R: Read>(reader: R) -> DecodeResult<ArtDecoder<R>> {
        ArtDecoder::new(reader)
    }

    /// Encode a collection of [`Image`]s into an ART stream.
    pub fn encode<W: Write>(writer: W, images: &[Image]) -> io::Result<()> {
        let entries: Vec<(ImageHeader, usize)> = images
            .iter()
            .map(|img| (img.header.clone(), img.data.len()))
            .collect();
        let mut encoder = ArtEncoder::new(writer, entries)?;
        for img in images {
            encoder.write_image(img.data.as_slice())?;
        }
        Ok(())
    }
}
