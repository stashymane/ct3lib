use crate::art::compression::{swizzle_rect, unswizzle_rect, Compression};
use crate::art::decoder::DecodeEntry;
use crate::art::image::Image;
use crate::art::image_header::ImageHeader;
use dds::header::{
    Dx9Header, Dx9PixelFormat, FourCC, Header, MaskPixelFormat, PixelFormatFlags, RgbBitCount,
};
use std::io::{self, Cursor, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DdsError {
    #[error("An IO error has occurred")]
    IoError(#[from] io::Error),
    #[error("Unsupported or unrecognized DDS pixel format")]
    UnsupportedFormat,
    #[error("DDS header is invalid: {0}")]
    InvalidHeader(String),
}

pub enum CompressionError {
    UnsupportedFormat,
}

/// Normalize raw ART image data into standard (top-to-bottom, unswizzled) form
fn normalize_for_dds(
    compression: Compression,
    data: &[u8],
    width: usize,
    height: usize,
) -> Vec<u8> {
    match compression {
        // Swizzled uncompressed formats: unswizzle only
        Compression::A4R4G4B4 => unswizzle_rect(data, width, height, 2),
        Compression::A8 => unswizzle_rect(data, width, height, 1),
        Compression::L8 => unswizzle_rect(data, width, height, 1),
        Compression::A8L8 => unswizzle_rect(data, width, height, 2),
        Compression::A1R5G5B5 => unswizzle_rect(data, width, height, 2),
        Compression::R5G6B5 => unswizzle_rect(data, width, height, 2),
        Compression::A8R8G8B8 => unswizzle_rect(data, width, height, 4),
        Compression::X8R8G8B8 => unswizzle_rect(data, width, height, 4),
        Compression::V8U8 => unswizzle_rect(data, width, height, 2),
        // DXT formats are stored upside-down in ART: flip vertically per row within each block
        Compression::DXT1 => flip_dxt(data, width, height, DxtVariant::Dxt1),
        Compression::DXT2 | Compression::DXT3 => flip_dxt(data, width, height, DxtVariant::Dxt3),
        Compression::DXT4 | Compression::DXT5 => flip_dxt(data, width, height, DxtVariant::Dxt5),
        // These formats have no swizzle/flip in ART storage: pass through as-is
        Compression::L6V5U5
        | Compression::UYVY
        | Compression::D24S8
        | Compression::D16
        | Compression::P8 { .. }
        | Compression::UNKNOWN => data.to_vec(),
    }
}

/// Denormalize DDS data back into ART storage form (re-swizzle / flip).
fn denormalize_from_dds(
    compression: Compression,
    data: &[u8],
    width: usize,
    height: usize,
) -> Vec<u8> {
    match compression {
        Compression::A4R4G4B4 => swizzle_rect(data, width, height, 2),
        Compression::A8 => swizzle_rect(data, width, height, 1),
        Compression::L8 => swizzle_rect(data, width, height, 1),
        Compression::A8L8 => swizzle_rect(data, width, height, 2),
        Compression::A1R5G5B5 => swizzle_rect(data, width, height, 2),
        Compression::R5G6B5 => swizzle_rect(data, width, height, 2),
        Compression::A8R8G8B8 => swizzle_rect(data, width, height, 4),
        Compression::X8R8G8B8 => swizzle_rect(data, width, height, 4),
        Compression::V8U8 => swizzle_rect(data, width, height, 2),
        Compression::DXT1 => flip_dxt(data, width, height, DxtVariant::Dxt1),
        Compression::DXT2 | Compression::DXT3 => flip_dxt(data, width, height, DxtVariant::Dxt3),
        Compression::DXT4 | Compression::DXT5 => flip_dxt(data, width, height, DxtVariant::Dxt5),
        Compression::L6V5U5
        | Compression::UYVY
        | Compression::D24S8
        | Compression::D16
        | Compression::P8 { .. }
        | Compression::UNKNOWN => data.to_vec(),
    }
}

/// Flip a DXT-compressed surface vertically: reverse block-row order and
/// reverse the pixel rows within each block.
fn flip_dxt(data: &[u8], width: usize, height: usize, variant: DxtVariant) -> Vec<u8> {
    let block_bytes = match variant {
        DxtVariant::Dxt1 => 8,
        _ => 16,
    };
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let row_bytes = blocks_x * block_bytes;
    let mut out = vec![0u8; blocks_y * row_bytes];
    for row in 0..blocks_y {
        let src_row = blocks_y - 1 - row;
        let src_base = src_row * row_bytes;
        let dst_base = row * row_bytes;
        for bx in 0..blocks_x {
            let src = src_base + bx * block_bytes;
            let dst = dst_base + bx * block_bytes;
            let block = &data[src..src + block_bytes];
            let flipped = variant.flip_block(block);
            out[dst..dst + block_bytes].copy_from_slice(&flipped);
        }
    }
    out
}

enum DxtVariant {
    Dxt1,
    Dxt3,
    Dxt5,
}

impl DxtVariant {
    fn flip_block(&self, block: &[u8]) -> Vec<u8> {
        match self {
            DxtVariant::Dxt1 => {
                let mut out = block.to_vec();
                out[4..8].reverse();
                out
            }
            DxtVariant::Dxt3 => {
                let mut out = block.to_vec();
                // Flip alpha rows: swap row0↔row3 and row1↔row2 (each row = 2 bytes)
                out.swap(0, 6);
                out.swap(1, 7); // row0 ↔ row3
                out.swap(2, 4);
                out.swap(3, 5); // row1 ↔ row2
                // Flip color rows
                out[12..16].reverse();
                out
            }
            DxtVariant::Dxt5 => {
                let mut out = block.to_vec();
                // Extract the 48-bit index field as a u64 (little-endian, bits 0-47)
                let idx = (block[2] as u64)
                    | ((block[3] as u64) << 8)
                    | ((block[4] as u64) << 16)
                    | ((block[5] as u64) << 24)
                    | ((block[6] as u64) << 32)
                    | ((block[7] as u64) << 40);
                // Each row is 12 bits (4 pixels × 3 bits)
                let row0 = (idx) & 0xFFF;
                let row1 = (idx >> 12) & 0xFFF;
                let row2 = (idx >> 24) & 0xFFF;
                let row3 = (idx >> 36) & 0xFFF;
                // Reverse row order
                let flipped = row3 | (row2 << 12) | (row1 << 24) | (row0 << 36);
                out[2] = (flipped) as u8;
                out[3] = (flipped >> 8) as u8;
                out[4] = (flipped >> 16) as u8;
                out[5] = (flipped >> 24) as u8;
                out[6] = (flipped >> 32) as u8;
                out[7] = (flipped >> 40) as u8;
                // Flip color rows
                out[12..16].reverse();
                out
            }
        }
    }
}

fn make_dds_header(
    compression: Compression,
    width: u16,
    height: u16,
    mip_count: u16,
) -> Option<Header> {
    let pf = compression.try_into().ok()?;
    let mut header = Dx9Header::new_image(width as u32, height as u32, pf);
    if mip_count > 1 {
        header = header.with_mipmap_count(std::num::NonZeroU32::new(mip_count as u32).unwrap());
    }
    Some(Header::Dx9(header))
}

impl Image {
    /// Normalize the raw image data and wrap it in a DDS container.
    /// Only the base mip level is included in the DDS file.
    pub fn to_dds(&self) -> Option<Vec<u8>> {
        let w = self.header.width as usize;
        let h = self.header.height as usize;
        let compression = self.header.compression;

        let header = make_dds_header(
            compression,
            self.header.width,
            self.header.height,
            self.header.mip_count,
        )?;

        // Collect all mip levels, normalized
        let mut normalized_data: Vec<u8> = Vec::new();
        let mut mw = w;
        let mut mh = h;
        let mut offset = 0usize;
        for _ in 0..self.header.mip_count {
            let mip_size = compression.base_mip_size(mw, mh);
            let end = (offset + mip_size).min(self.data.len());
            let mip_slice = &self.data[offset..end];
            normalized_data.extend(normalize_for_dds(compression, mip_slice, mw, mh));
            offset += mip_size;
            if offset >= self.data.len() {
                break;
            }
            mw = (mw / 2).max(1);
            mh = (mh / 2).max(1);
        }

        let mut buf = Vec::new();
        header.write(&mut buf).ok()?;
        buf.extend_from_slice(&normalized_data);
        Some(buf)
    }

    /// Parse a DDS file from bytes and re-encode it into this library's ART format.
    pub fn from_dds_bytes(
        data: &[u8],
        compression: Compression,
        mip_count: u16,
    ) -> Result<Self, DdsError> {
        Self::from_dds_reader(Cursor::new(data), compression, mip_count)
    }

    /// Parse a DDS file from disk and re-encode it into this library's ART format.
    pub fn from_dds(
        path: &Path,
        compression: Compression,
        mip_count: u16,
    ) -> Result<Self, DdsError> {
        let data = std::fs::read(path)?;
        Self::from_dds_bytes(&data, compression, mip_count)
    }

    fn from_dds_reader<R: Read>(
        mut reader: R,
        compression: Compression,
        mip_count: u16,
    ) -> Result<Self, DdsError> {
        // Header::read reads magic by default
        let options = dds::header::ParseOptions::default();
        let header = Header::read(&mut reader, &options)
            .map_err(|e| DdsError::InvalidHeader(e.to_string()))?;

        let width = header.width() as u16;
        let height = header.height() as u16;

        // Read remaining bytes as pixel data
        let mut pixel_data = Vec::new();
        reader.read_to_end(&mut pixel_data)?;

        // Re-encode each mip level from normalized DDS data back into ART format
        let mut art_data: Vec<u8> = Vec::new();
        let mut mw = width as usize;
        let mut mh = height as usize;
        let mut offset = 0usize;
        for _ in 0..mip_count {
            let mip_size = compression.base_mip_size(mw, mh);
            let end = (offset + mip_size).min(pixel_data.len());
            let mip_slice = if end > offset {
                &pixel_data[offset..end]
            } else {
                &[]
            };
            art_data.extend(denormalize_from_dds(compression, mip_slice, mw, mh));
            offset += mip_size;
            if offset >= pixel_data.len() {
                break;
            }
            mw = (mw / 2).max(1);
            mh = (mh / 2).max(1);
        }

        let mut image = Image {
            header: ImageHeader {
                width,
                height,
                size: 0,
                compression,
                mip_count,
            },
            data: art_data,
        };
        image.header.size = image.header.total_data_size() as u32;
        Ok(image)
    }

    /// Detect the `Compression` stored in a DDS file's header without fully decoding it.
    /// Returns `None` if the format cannot be mapped to a known `Compression`.
    pub fn compression_from_dds_bytes(data: &[u8]) -> Option<Compression> {
        let mut cursor = Cursor::new(data);
        let options = dds::header::ParseOptions::default();
        // Header::read reads magic by default
        let header = Header::read(&mut cursor, &options).ok()?;
        match header {
            Header::Dx9(dx9) => dx9.pixel_format.try_into().ok(),
            Header::Dx10(_) => None,
        }
    }
}

impl DecodeEntry {
    /// Normalize and wrap the raw image data as a DDS file in memory.
    pub fn to_dds(&self) -> Option<Vec<u8>> {
        self.as_image().to_dds()
    }
}

impl TryFrom<Compression> for Dx9PixelFormat {
    type Error = CompressionError;

    fn try_from(compression: Compression) -> Result<Self, Self::Error> {
        Ok(match compression {
            Compression::DXT1 => Dx9PixelFormat::FourCC(FourCC::DXT1),
            Compression::DXT2 => Dx9PixelFormat::FourCC(FourCC::DXT2),
            Compression::DXT3 => Dx9PixelFormat::FourCC(FourCC::DXT3),
            Compression::DXT4 => Dx9PixelFormat::FourCC(FourCC::DXT4),
            Compression::DXT5 => Dx9PixelFormat::FourCC(FourCC::DXT5),

            Compression::A8R8G8B8 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::RGBA,
                rgb_bit_count: RgbBitCount::Count32,
                r_bit_mask: 0x00ff_0000,
                g_bit_mask: 0x0000_ff00,
                b_bit_mask: 0x0000_00ff,
                a_bit_mask: 0xff00_0000,
            }),
            Compression::X8R8G8B8 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::RGB,
                rgb_bit_count: RgbBitCount::Count32,
                r_bit_mask: 0x00ff_0000,
                g_bit_mask: 0x0000_ff00,
                b_bit_mask: 0x0000_00ff,
                a_bit_mask: 0x0000_0000,
            }),
            Compression::R5G6B5 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::RGB,
                rgb_bit_count: RgbBitCount::Count16,
                r_bit_mask: 0xf800,
                g_bit_mask: 0x07e0,
                b_bit_mask: 0x001f,
                a_bit_mask: 0x0000,
            }),
            Compression::A1R5G5B5 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::RGBA,
                rgb_bit_count: RgbBitCount::Count16,
                r_bit_mask: 0x7c00,
                g_bit_mask: 0x03e0,
                b_bit_mask: 0x001f,
                a_bit_mask: 0x8000,
            }),
            Compression::A4R4G4B4 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::RGBA,
                rgb_bit_count: RgbBitCount::Count16,
                r_bit_mask: 0x0f00,
                g_bit_mask: 0x00f0,
                b_bit_mask: 0x000f,
                a_bit_mask: 0xf000,
            }),
            Compression::A8 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::ALPHA,
                rgb_bit_count: RgbBitCount::Count8,
                r_bit_mask: 0x00,
                g_bit_mask: 0x00,
                b_bit_mask: 0x00,
                a_bit_mask: 0xff,
            }),
            Compression::L8 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::LUMINANCE,
                rgb_bit_count: RgbBitCount::Count8,
                r_bit_mask: 0xff,
                g_bit_mask: 0x00,
                b_bit_mask: 0x00,
                a_bit_mask: 0x00,
            }),
            Compression::A8L8 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::LUMINANCE_ALPHA,
                rgb_bit_count: RgbBitCount::Count16,
                r_bit_mask: 0x00ff,
                g_bit_mask: 0x0000,
                b_bit_mask: 0x0000,
                a_bit_mask: 0xff00,
            }),
            Compression::V8U8 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::from_bits_retain(0x00080000), // DDPF_BUMPDUDV
                rgb_bit_count: RgbBitCount::Count16,
                r_bit_mask: 0x00ff,
                g_bit_mask: 0xff00,
                b_bit_mask: 0x0000,
                a_bit_mask: 0x0000,
            }),
            Compression::L6V5U5 => Dx9PixelFormat::Mask(MaskPixelFormat {
                // DDPF_BUMPDUDV | DDPF_BUMPLUMINANCE
                flags: PixelFormatFlags::from_bits_retain(0x00040000 | 0x00080000),
                rgb_bit_count: RgbBitCount::Count16,
                r_bit_mask: 0x001f, // U: bits 0-4
                g_bit_mask: 0x03e0, // V: bits 5-9
                b_bit_mask: 0xfc00, // L: bits 10-15
                a_bit_mask: 0x0000,
            }),
            Compression::UYVY => Dx9PixelFormat::FourCC(FourCC::UYVY),
            Compression::D24S8 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::from_bits_retain(0x00000002), // DDPF_ALPHA (depth/stencil)
                rgb_bit_count: RgbBitCount::Count32,
                r_bit_mask: 0x00ff_ffff, // depth in lower 24 bits
                g_bit_mask: 0x0000_0000,
                b_bit_mask: 0x0000_0000,
                a_bit_mask: 0xff00_0000, // stencil in upper 8 bits
            }),
            Compression::D16 => Dx9PixelFormat::Mask(MaskPixelFormat {
                flags: PixelFormatFlags::from_bits_retain(0x00000002), // DDPF_ALPHA (depth)
                rgb_bit_count: RgbBitCount::Count16,
                r_bit_mask: 0xffff,
                g_bit_mask: 0x0000,
                b_bit_mask: 0x0000,
                a_bit_mask: 0x0000,
            }),
            Compression::P8 { n_palettes } => Dx9PixelFormat::Mask(MaskPixelFormat {
                // Store palette count in the unused g_bit_mask field for round-trip
                flags: PixelFormatFlags::from_bits_retain(0x00000020), // DDPF_PALETTEINDEXED8
                rgb_bit_count: RgbBitCount::Count8,
                r_bit_mask: 0x00,
                g_bit_mask: n_palettes as u32,
                b_bit_mask: 0x00,
                a_bit_mask: 0x00,
            }),
            Compression::UNKNOWN => return Err(CompressionError::UnsupportedFormat),
        })
    }
}

impl TryFrom<Dx9PixelFormat> for Compression {
    type Error = CompressionError;

    fn try_from(format: Dx9PixelFormat) -> Result<Self, Self::Error> {
        Ok(match format {
            Dx9PixelFormat::FourCC(fcc) => match fcc {
                FourCC::DXT1 => Compression::DXT1,
                FourCC::DXT2 => Compression::DXT2,
                FourCC::DXT3 => Compression::DXT3,
                FourCC::DXT4 => Compression::DXT4,
                FourCC::DXT5 => Compression::DXT5,
                _ => return Err(CompressionError::UnsupportedFormat),
            },
            Dx9PixelFormat::Mask(m) => {
                let flags = m.flags;
                let bumpdudv = PixelFormatFlags::from_bits_retain(0x00080000);
                let bumpluminance = PixelFormatFlags::from_bits_retain(0x00040000);
                let paletteindexed8 = PixelFormatFlags::from_bits_retain(0x00000020);

                if flags.contains(paletteindexed8) {
                    let n_palettes = m.g_bit_mask as u8;
                    Compression::P8 { n_palettes }
                } else if flags.contains(bumpdudv) && flags.contains(bumpluminance) {
                    Compression::L6V5U5
                } else if flags.contains(bumpdudv) {
                    Compression::V8U8
                } else if flags.contains(PixelFormatFlags::ALPHA)
                    && !flags.contains(PixelFormatFlags::RGB)
                {
                    // Distinguish D24S8 (32-bit) from D16 (16-bit) and A8
                    match m.rgb_bit_count {
                        RgbBitCount::Count32 => Compression::D24S8,
                        RgbBitCount::Count16 if m.r_bit_mask == 0xffff => Compression::D16,
                        RgbBitCount::Count8 => Compression::A8,
                        _ => return Err(CompressionError::UnsupportedFormat),
                    }
                } else if flags.contains(PixelFormatFlags::LUMINANCE) {
                    if flags.contains(PixelFormatFlags::ALPHAPIXELS) {
                        Compression::A8L8
                    } else {
                        Compression::L8
                    }
                } else if flags.contains(PixelFormatFlags::RGB) {
                    match m.rgb_bit_count {
                        RgbBitCount::Count32 => {
                            if m.a_bit_mask != 0 {
                                Compression::A8R8G8B8
                            } else {
                                Compression::X8R8G8B8
                            }
                        }
                        RgbBitCount::Count16 => {
                            if m.a_bit_mask == 0x8000 {
                                Compression::A1R5G5B5
                            } else if m.a_bit_mask == 0xf000 {
                                Compression::A4R4G4B4
                            } else if m.a_bit_mask == 0 && m.r_bit_mask == 0xf800 {
                                Compression::R5G6B5
                            } else {
                                return Err(CompressionError::UnsupportedFormat);
                            }
                        }
                        _ => return Err(CompressionError::UnsupportedFormat),
                    }
                } else {
                    return Err(CompressionError::UnsupportedFormat);
                }
            }
        })
    }
}
