use crate::art::compression::util::swizzle_rect;
use crate::art::compression::Compression;
use dds::header::{Dx9Header, Dx9PixelFormat, FourCC, Header};
use dds::{ColorFormat, Encoder, Format, ImageView, Size};

impl Compression {
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
            Compression::A8 => {
                let linear: Vec<u8> = source.chunks_exact(4).map(|p| p[3]).collect();
                swizzle_rect(&linear, width, height, 1)
            }
            Compression::L8 => {
                let linear: Vec<u8> = source
                    .chunks_exact(4)
                    .map(|p| {
                        let r = p[0] as u32;
                        let g = p[1] as u32;
                        let b = p[2] as u32;
                        ((r * 299 + g * 587 + b * 114 + 500) / 1000) as u8
                    })
                    .collect();
                swizzle_rect(&linear, width, height, 1)
            }
            Compression::A8L8 => {
                let linear: Vec<u8> = source
                    .chunks_exact(4)
                    .flat_map(|p| {
                        let r = p[0] as u32;
                        let g = p[1] as u32;
                        let b = p[2] as u32;
                        let l = ((r * 299 + g * 587 + b * 114 + 500) / 1000) as u8;
                        let a = p[3];
                        [l, a]
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
            Compression::DXT1 => encode_dxt(source, width, height, Format::BC1_UNORM),
            Compression::X8R8G8B8 => {
                let linear: Vec<u8> = source
                    .chunks_exact(4)
                    .flat_map(|p| {
                        let r = p[0];
                        let g = p[1];
                        let b = p[2];
                        [b, g, r, 0xff]
                    })
                    .collect();
                swizzle_rect(&linear, width, height, 4)
            }
            Compression::DXT2 => {
                encode_dxt(source, width, height, Format::BC2_UNORM_PREMULTIPLIED_ALPHA)
            }
            Compression::DXT3 => encode_dxt(source, width, height, Format::BC2_UNORM),
            Compression::DXT4 => {
                encode_dxt(source, width, height, Format::BC3_UNORM_PREMULTIPLIED_ALPHA)
            }
            Compression::DXT5 => encode_dxt(source, width, height, Format::BC3_UNORM),
            Compression::P8 { n_palettes } => {
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
            Compression::V8U8
            | Compression::L6V5U5
            | Compression::UYVY
            | Compression::D24S8
            | Compression::D16
            | Compression::UNKNOWN => todo!("encode not supported for {self:?}"),
        }
    }
}

/// Encode RGBA8 pixels (flipped vertically) into raw DXT block data using the dds crate.
fn encode_dxt(source: &[u8], width: usize, height: usize, format: Format) -> Vec<u8> {
    // ART stores DXT upside-down, so flip before encoding
    let flipped: Vec<u8> = source
        .chunks_exact(width * 4)
        .rev()
        .flat_map(|r| r.iter().copied())
        .collect();

    // Use a Dx9 FourCC header to guarantee a fixed 128-byte header (magic + dx9)
    let four_cc = match format {
        Format::BC1_UNORM => FourCC::DXT1,
        Format::BC2_UNORM => FourCC::DXT3,
        Format::BC2_UNORM_PREMULTIPLIED_ALPHA => FourCC::DXT2,
        Format::BC3_UNORM => FourCC::DXT5,
        Format::BC3_UNORM_PREMULTIPLIED_ALPHA => FourCC::DXT4,
        _ => panic!("encode_dxt: unsupported format {format:?}"),
    };
    let header = Header::Dx9(Dx9Header::new_image(
        width as u32,
        height as u32,
        Dx9PixelFormat::FourCC(four_cc),
    ));
    let mut dds_bytes: Vec<u8> = Vec::new();
    let mut encoder =
        Encoder::new(&mut dds_bytes, format, &header).expect("failed to create DXT encoder");
    let view = ImageView::new(
        &flipped,
        Size::new(width as u32, height as u32),
        ColorFormat::RGBA_U8,
    )
    .expect("failed to create image view");
    encoder
        .write_surface(view)
        .expect("failed to encode DXT surface");
    encoder.finish().expect("failed to finish DXT encoding");

    // Strip DDS magic (4 bytes) + Dx9 header (124 bytes) = 128 bytes
    dds_bytes[128..].to_vec()
}
