use crate::data::compression_util::{swizzle_rect, unswizzle_rect};
use serde::{Deserialize, Serialize};
use texpresso::{Format, Params};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(usize)]
pub enum Compression {
    A1R5G5B5 = 0,
    A4R4G4B4 = 1,
    DXT3Alt = 3,
    X1R5G5B5 = 4,
    R5G6B5 = 5,
    A8R8G8B8 = 6,
    DXT1 = 10,
    DXT3 = 11,
    /// Paletted (indexed colour). The `n_palettes` field records how many
    /// 256-entry RGBA8 palettes are appended after the index data.
    Paletted {
        n_palettes: u8,
    } = 15,
}

impl Compression {
    pub fn from_u32(value: u32) -> Option<Self> {
        let hi = (value >> 16) as u16;
        let base_type = (hi & 0xff) as u8;
        let upper = (hi >> 8) as u8;
        // Paletted format: lower byte of hi word == 0x0f, upper byte == palette count
        if base_type == 0x0f {
            return Some(Compression::Paletted { n_palettes: upper });
        }
        match hi {
            0 => Some(Compression::A1R5G5B5),
            1 => Some(Compression::A4R4G4B4),
            3 => Some(Compression::DXT3Alt),
            4 => Some(Compression::X1R5G5B5),
            5 => Some(Compression::R5G6B5),
            6 => Some(Compression::A8R8G8B8),
            10 => Some(Compression::DXT1),
            11 => Some(Compression::DXT3),
            _ => None,
        }
    }

    pub fn to_u32(self) -> u32 {
        let hi: u32 = match self {
            Compression::A1R5G5B5 => 0,
            Compression::A4R4G4B4 => 1,
            Compression::DXT3Alt => 3,
            Compression::X1R5G5B5 => 4,
            Compression::R5G6B5 => 5,
            Compression::A8R8G8B8 => 6,
            Compression::DXT1 => 10,
            Compression::DXT3 => 11,
            Compression::Paletted { n_palettes } => ((n_palettes as u32) << 8) | 0x0f,
        };
        (hi << 16) | 1
    }

    /// Returns the byte size of the base (mip 0) image only, ignoring any mip chain.
    pub fn base_mip_size(self, width: usize, height: usize) -> usize {
        match self {
            Compression::A4R4G4B4
            | Compression::X1R5G5B5
            | Compression::A1R5G5B5
            | Compression::R5G6B5 => width * height * 2,
            Compression::A8R8G8B8 => width * height * 4,
            Compression::DXT1 => Format::Bc1.compressed_size(width, height),
            Compression::DXT3 | Compression::DXT3Alt => Format::Bc2.compressed_size(width, height),
            Compression::Paletted { n_palettes } => width * height + n_palettes as usize * 256 * 4,
        }
    }

    pub fn encode(self, source: &[u8], width: usize, height: usize) -> Vec<u8> {
        match self {
            Compression::A4R4G4B4 => {
                let linear: Vec<u8> = source
                    .chunks_exact(4)
                    .flat_map(|p| {
                        let r = (p[0] as u16 * 15 / 255) & 0xf;
                        let g = (p[1] as u16 * 15 / 255) & 0xf;
                        let b = (p[2] as u16 * 15 / 255) & 0xf;
                        let a = (p[3] as u16 * 15 / 255) & 0xf;
                        let v: u16 = (a << 12) | (r << 8) | (g << 4) | b;
                        v.to_le_bytes()
                    })
                    .collect();
                swizzle_rect(&linear, width, height, 2)
            }
            Compression::X1R5G5B5 => {
                let linear: Vec<u8> = source
                    .chunks_exact(4)
                    .flat_map(|p| {
                        let r = ((p[0] as u16 * 31 + 127) / 255) & 0x1f;
                        let g = ((p[1] as u16 * 31 + 127) / 255) & 0x1f;
                        let b = ((p[2] as u16 * 31 + 127) / 255) & 0x1f;
                        let v: u16 = (r << 10) | (g << 5) | b;
                        v.to_le_bytes()
                    })
                    .collect();
                swizzle_rect(&linear, width, height, 2)
            }
            Compression::A1R5G5B5 => {
                let linear: Vec<u8> = source
                    .chunks_exact(4)
                    .flat_map(|p| {
                        let r = ((p[0] as u16 * 31 + 127) / 255) & 0x1f;
                        let g = ((p[1] as u16 * 31 + 127) / 255) & 0x1f;
                        let b = ((p[2] as u16 * 31 + 127) / 255) & 0x1f;
                        let a: u16 = if p[3] >= 128 { 1 } else { 0 };
                        let v: u16 = (a << 15) | (r << 10) | (g << 5) | b;
                        v.to_le_bytes()
                    })
                    .collect();
                swizzle_rect(&linear, width, height, 2)
            }
            Compression::R5G6B5 => {
                let linear: Vec<u8> = source
                    .chunks_exact(4)
                    .flat_map(|p| {
                        let r = ((p[0] as u16 * 31 + 127) / 255) & 0x1f;
                        let g = ((p[1] as u16 * 63 + 127) / 255) & 0x3f;
                        let b = ((p[2] as u16 * 31 + 127) / 255) & 0x1f;
                        let v: u16 = (r << 11) | (g << 5) | b;
                        v.to_le_bytes()
                    })
                    .collect();
                swizzle_rect(&linear, width, height, 2)
            }
            Compression::A8R8G8B8 => {
                let linear: Vec<u8> = source
                    .chunks_exact(4)
                    .flat_map(|p| {
                        let r = p[0];
                        let g = p[1];
                        let b = p[2];
                        let a = p[3];
                        [b, g, r, a]
                    })
                    .collect();
                swizzle_rect(&linear, width, height, 4)
            }
            Compression::DXT1 => {
                let flipped: Vec<u8> = source
                    .chunks_exact(width * 4)
                    .rev()
                    .flat_map(|r| r.iter().copied())
                    .collect();
                let mut out = vec![0u8; Format::Bc1.compressed_size(width, height)];
                Format::Bc1.compress(&flipped, width, height, Params::default(), &mut out);
                out
            }
            Compression::DXT3 | Compression::DXT3Alt => {
                let flipped: Vec<u8> = source
                    .chunks_exact(width * 4)
                    .rev()
                    .flat_map(|r| r.iter().copied())
                    .collect();
                let mut out = vec![0u8; Format::Bc2.compressed_size(width, height)];
                Format::Bc2.compress(&flipped, width, height, Params::default(), &mut out);
                out
            }
            Compression::Paletted { n_palettes } => {
                let n = n_palettes as usize;
                let pixel_count = width * height;
                // Encode: write index bytes (Xbox-swizzled), then palettes.
                // For a round-trip from RGBA we build a single palette and write n copies.
                let mut palette = vec![0u8; 256 * 4];
                let mut indices = vec![0u8; pixel_count];
                let mut color_map: std::collections::HashMap<[u8; 4], u8> =
                    std::collections::HashMap::new();
                let mut next_idx: u8 = 0;
                for (i, pixel) in source.chunks_exact(4).enumerate() {
                    let key = [pixel[0], pixel[1], pixel[2], pixel[3]];
                    let idx = *color_map.entry(key).or_insert_with(|| {
                        let idx = next_idx;
                        let off = idx as usize * 4;
                        palette[off] = key[0];
                        palette[off + 1] = key[1];
                        palette[off + 2] = key[2];
                        palette[off + 3] = key[3];
                        next_idx = next_idx.wrapping_add(1);
                        idx
                    });
                    indices[i] = idx;
                }
                let swizzled = swizzle_rect(&indices, width, height, 1);
                let mut out = swizzled;
                for _ in 0..n {
                    out.extend_from_slice(&palette);
                }
                out
            }
        }
    }

    pub fn decode(self, source: &[u8], width: usize, height: usize) -> Vec<u8> {
        match self {
            Compression::A4R4G4B4 => {
                let unswiz = unswizzle_rect(source, width, height, 2);
                unswiz
                    .chunks_exact(2)
                    .flat_map(|p| {
                        let v = u16::from_le_bytes([p[0], p[1]]);
                        let a = (((v >> 12) & 0xf) * 255 / 15) as u8;
                        let r = (((v >> 8) & 0xf) * 255 / 15) as u8;
                        let g = (((v >> 4) & 0xf) * 255 / 15) as u8;
                        let b = ((v & 0xf) * 255 / 15) as u8;
                        [r, g, b, a]
                    })
                    .collect()
            }
            Compression::X1R5G5B5 => {
                let unswiz = unswizzle_rect(source, width, height, 2);
                unswiz
                    .chunks_exact(2)
                    .flat_map(|p| {
                        let v = u16::from_le_bytes([p[0], p[1]]);
                        let r = (((v >> 10) & 0x1f) * 255 / 31) as u8;
                        let g = (((v >> 5) & 0x1f) * 255 / 31) as u8;
                        let b = ((v & 0x1f) * 255 / 31) as u8;
                        [r, g, b, 255u8]
                    })
                    .collect()
            }
            Compression::A1R5G5B5 => {
                let unswiz = unswizzle_rect(source, width, height, 2);
                unswiz
                    .chunks_exact(2)
                    .flat_map(|p| {
                        let v = u16::from_le_bytes([p[0], p[1]]);
                        let a = if v >> 15 != 0 { 255u8 } else { 0u8 };
                        let r = (((v >> 10) & 0x1f) * 255 / 31) as u8;
                        let g = (((v >> 5) & 0x1f) * 255 / 31) as u8;
                        let b = ((v & 0x1f) * 255 / 31) as u8;
                        [r, g, b, a]
                    })
                    .collect()
            }
            Compression::R5G6B5 => {
                let unswiz = unswizzle_rect(source, width, height, 2);
                unswiz
                    .chunks_exact(2)
                    .flat_map(|p| {
                        let v = u16::from_le_bytes([p[0], p[1]]);
                        let r = (((v >> 11) & 0x1f) * 255 / 31) as u8;
                        let g = (((v >> 5) & 0x3f) * 255 / 63) as u8;
                        let b = ((v & 0x1f) * 255 / 31) as u8;
                        [r, g, b, 255u8]
                    })
                    .collect()
            }
            Compression::A8R8G8B8 => {
                let unswiz = unswizzle_rect(source, width, height, 4);
                unswiz
                    .chunks_exact(4)
                    .flat_map(|p| {
                        let b = p[0];
                        let g = p[1];
                        let r = p[2];
                        let a = p[3];
                        [r, g, b, a]
                    })
                    .collect()
            }
            Compression::DXT1 => {
                let expected = Format::Bc1.compressed_size(width, height);
                let mut padded;
                let src = if source.len() < expected {
                    padded = vec![0u8; expected];
                    padded[..source.len()].copy_from_slice(source);
                    &padded[..]
                } else {
                    source
                };
                let mut out = vec![0u8; width * height * 4];
                Format::Bc1.decompress(src, width, height, &mut out);
                out.chunks_exact(width * 4)
                    .rev()
                    .flat_map(|r| r.iter().copied())
                    .collect()
            }
            Compression::DXT3 | Compression::DXT3Alt => {
                let expected = Format::Bc2.compressed_size(width, height);
                let mut padded;
                let src = if source.len() < expected {
                    padded = vec![0u8; expected];
                    padded[..source.len()].copy_from_slice(source);
                    &padded[..]
                } else {
                    source
                };
                let mut out = vec![0u8; width * height * 4];
                Format::Bc2.decompress(src, width, height, &mut out);
                out.chunks_exact(width * 4)
                    .rev()
                    .flat_map(|r| r.iter().copied())
                    .collect()
            }
            Compression::Paletted { n_palettes: _ } => {
                let pixel_count = width * height;
                // Layout: pixel_count index bytes (Xbox-swizzled), then one or more 256-entry RGBA8 palettes.
                // Use the first palette (palette 0) for decoding.
                let palette_offset = pixel_count;
                let indices = unswizzle_rect(&source[..pixel_count], width, height, 1);
                indices
                    .iter()
                    .flat_map(|&idx| {
                        let off = palette_offset + idx as usize * 4;
                        [
                            source[off],
                            source[off + 1],
                            source[off + 2],
                            source[off + 3],
                        ]
                    })
                    .collect()
            }
        }
    }
}
