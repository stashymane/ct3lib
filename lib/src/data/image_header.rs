use crate::data::{Compression, DecodeError, DecodeResult};
use crate::util::{read_u16, read_u32};
use crate::MAGIC;
use std::io::Read;

/// Image metadata without the raw pixel data — used for streaming APIs.
#[derive(Debug, Clone)]
pub struct ImageHeader {
    pub width: u16,
    pub height: u16,
    pub size: u32,
    pub compression: Compression,
    /// Number of mipmap levels (1 = base image only, >1 = mip chain).
    /// This is the low 16 bits of the raw compression u32 from the file.
    pub mip_count: u16,
}

impl ImageHeader {
    /// Total byte size of all mip levels as stored in the file.
    pub fn total_data_size(&self) -> usize {
        let mut size = 0;
        let mut w = self.width as usize;
        let mut h = self.height as usize;
        for _ in 0..self.mip_count {
            size += self.compression.base_mip_size(w, h);
            if w == 1 && h == 1 {
                break;
            }
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }
        size
    }

    /// The raw u32 compression field value to write into the file.
    pub fn comp_u32(&self) -> u32 {
        (self.compression.to_u32() & 0xffff_0000) | self.mip_count as u32
    }

    pub fn read_from<R>(reader: &mut R) -> DecodeResult<ImageHeader>
    where
        R: Read,
    {
        let magic = read_u32(reader)?;
        if magic != MAGIC {
            return Err(DecodeError::InvalidMagic {
                offset: 0,
                got: magic,
            });
        }

        let width = read_u16(reader)?;
        let height = read_u16(reader)?;
        let size = read_u32(reader)?;
        let comp_raw = read_u32(reader)?;

        let compression = Compression::from_u32(comp_raw)
            .ok_or(DecodeError::UnknownCompression { value: comp_raw })?;
        let mip_count = (comp_raw & 0xffff) as u16;

        Ok(ImageHeader {
            width,
            height,
            size,
            compression,
            mip_count,
        })
    }
}
