pub mod data;
pub mod png;
pub mod util;

use crate::data::{Image, ImageHeader};
pub use data::{ArtDecoder, ArtDecoderIter, ArtEncoder, DecodeEntry, DecodeError, DecodeResult};
use std::io::{self, Read, Write};

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
    /// use ct3lib::Art;
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
