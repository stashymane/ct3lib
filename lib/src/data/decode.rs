use crate::data::ImageHeader;
use crate::util::read_u32;
use std::io;
use std::io::{Read, Seek, SeekFrom, Take};
use thiserror::Error;

pub struct ArtDecoder<R: Read> {
    reader: R,
    offsets: Vec<usize>,
    remaining: usize,
    total: usize,
}

pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("invalid magic at offset {offset}: got {got}")]
    InvalidMagic { offset: usize, got: u32 },
    #[error("unknown compression type: {value}")]
    UnknownCompression { value: u32 },
    #[error("index {index} out of bounds")]
    IndexOutOfBounds { index: usize },
}

impl<R: Read> ArtDecoder<R> {
    /// Begin decoding an ART stream. Reads the count and pointer table immediately.
    pub fn new(mut reader: R) -> DecodeResult<Self> {
        let count = read_u32(&mut reader)? as usize;

        let mut offsets = Vec::with_capacity(count);
        for _ in 0..count {
            let offset = read_u32(&mut reader)? as usize;
            offsets.push(offset);
        }

        Ok(Self {
            reader,
            offsets,
            remaining: count,
            total: count,
        })
    }

    /// Total number of images in this ART file.
    pub fn len(&self) -> usize {
        self.total
    }

    pub fn current(&self) -> usize {
        self.total - self.remaining
    }

    pub fn is_empty(&self) -> bool {
        self.remaining == 0
    }

    pub fn skip(&mut self, count: usize) -> Result<(), DecodeError> {
        if count > self.remaining {
            return Err(DecodeError::IndexOutOfBounds {
                index: self.remaining + count,
            });
        }
        self.remaining -= count;
        Ok(())
    }

    /// Read the next image header and return it together with a reader limited
    /// to exactly the image's data bytes. The caller **must** fully consume or
    /// drop the `Take<R>` before calling `next_entry` again.
    pub fn next_entry(&mut self) -> DecodeResult<Option<(ImageHeader, Take<&mut R>)>> {
        if self.remaining == 0 {
            return Ok(None);
        }

        let header = ImageHeader::read_from(&mut self.reader)?;
        let data_reader = (&mut self.reader).take(header.size as u64);

        self.remaining -= 1;
        Ok(Some((header, data_reader)))
    }
}

impl<R: Read + Seek> ArtDecoder<R> {
    pub fn header_at(&mut self, index: usize) -> Result<ImageHeader, DecodeError> {
        let offset = self
            .offsets
            .get(index)
            .ok_or(DecodeError::IndexOutOfBounds { index })?;

        self.reader.seek(SeekFrom::Start(*offset as u64))?;
        let header = ImageHeader::read_from(&mut self.reader)?;

        Ok(header)
    }

    pub fn entry_at(&mut self, index: usize) -> Result<(ImageHeader, Take<&mut R>), DecodeError> {
        let offset = self
            .offsets
            .get(index)
            .ok_or(DecodeError::IndexOutOfBounds { index })?;

        self.reader.seek(SeekFrom::Start(*offset as u64))?;
        let header = ImageHeader::read_from(&mut self.reader)?;
        let data_reader = (&mut self.reader).take(header.size as u64);

        Ok((header, data_reader))
    }
}

/// An iterator over the entries of an [`ArtDecoder`].
///
/// Each item is a [`DecodeEntry`] which exposes the image header and allows
/// decoding the pixel data on demand via [`DecodeEntry::decode`].
pub struct ArtDecoderIter<R: Read> {
    decoder: ArtDecoder<R>,
}

/// A single entry produced by [`ArtDecoderIter`].
///
/// The header is available immediately; call [`DecodeEntry::decode`] to read
/// and decompress the pixel data.
pub struct DecodeEntry {
    pub header: ImageHeader,
    pub data: Vec<u8>,
}

impl DecodeEntry {
    fn as_image(&self) -> crate::data::image::Image {
        crate::data::image::Image {
            header: self.header.clone(),
            data: self.data.clone(),
        }
    }

    /// Decode the raw image data into RGBA8 pixels (row-major, top-to-bottom).
    pub fn decode(&self) -> Vec<u8> {
        self.as_image().decode()
    }

    /// Encode the decoded pixels as a PNG file in memory.
    pub fn to_png(&self) -> Vec<u8> {
        self.as_image().to_png()
    }
}

impl<R: Read> Iterator for ArtDecoderIter<R> {
    type Item = DecodeResult<DecodeEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.next_entry() {
            Ok(Some((header, mut data_reader))) => {
                let mut data = Vec::new();
                match data_reader.read_to_end(&mut data) {
                    Ok(_) => Some(Ok(DecodeEntry { header, data })),
                    Err(e) => Some(Err(DecodeError::Io(e))),
                }
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<R: Read> IntoIterator for ArtDecoder<R> {
    type Item = DecodeResult<DecodeEntry>;
    type IntoIter = ArtDecoderIter<R>;

    fn into_iter(self) -> Self::IntoIter {
        ArtDecoderIter { decoder: self }
    }
}
