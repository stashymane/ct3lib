use crate::data::{Compression, ImageHeader};
use crate::{read_u16, read_u32, DecodeError, MAGIC};
use std::io::{Read, Take};

/// Streaming ART decoder. Reads the file header eagerly; image data is read
/// on demand by consuming the `Take<R>` returned from [`ArtDecoder::next_entry`].
pub struct ArtDecoder<R: Read> {
    reader: R,
    remaining: usize,
}

impl<R: Read> ArtDecoder<R> {
    /// Begin decoding an ART stream. Reads the count and pointer table immediately.
    pub fn new(mut reader: R) -> Result<Self, DecodeError> {
        let count = read_u32(&mut reader)? as usize;
        // Read and discard the pointer table (not needed for sequential reading)
        let mut ptr_buf = vec![0u8; count * 4];
        reader.read_exact(&mut ptr_buf)?;
        Ok(Self {
            reader,
            remaining: count,
        })
    }

    /// Total number of images in this ART file.
    pub fn len(&self) -> usize {
        self.remaining
    }

    pub fn is_empty(&self) -> bool {
        self.remaining == 0
    }

    /// Read the next image header and return it together with a reader limited
    /// to exactly the image's data bytes. The caller **must** fully consume or
    /// drop the `Take<R>` before calling `next_entry` again.
    pub fn next_entry(&mut self) -> Result<Option<(ImageHeader, Take<&mut R>)>, DecodeError> {
        if self.remaining == 0 {
            return Ok(None);
        }

        let magic = read_u32(&mut self.reader)?;
        if magic != MAGIC {
            return Err(DecodeError::InvalidMagic {
                offset: 0,
                got: magic,
            });
        }

        let width = read_u16(&mut self.reader)?;
        let height = read_u16(&mut self.reader)?;
        let size = read_u32(&mut self.reader)? as u64;
        let comp_raw = read_u32(&mut self.reader)?;

        let compression = Compression::from_u32(comp_raw)
            .ok_or(DecodeError::UnknownCompression { value: comp_raw })?;
        let mip_count = (comp_raw & 0xffff) as u16;

        let header = ImageHeader {
            width,
            height,
            compression,
            mip_count,
        };
        // Use the on-disk size field (not the computed size) so the reader
        // stays correctly positioned even if the file's size differs.
        let data_reader = (&mut self.reader).take(size);
        self.remaining -= 1;
        Ok(Some((header, data_reader)))
    }
}
