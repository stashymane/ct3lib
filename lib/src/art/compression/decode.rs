use crate::art::compression::util::unswizzle_rect;
use crate::art::compression::Compression;
use dds::header::{Dx9Header, Dx9PixelFormat, FourCC, Header};
use dds::{ColorFormat, Decoder, Format, ImageViewMut, Size};
use std::io::Cursor;

impl Compression {
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
            Compression::A8 => {
                let unswiz = unswizzle_rect(source, width, height, 1);
                unswiz.iter().flat_map(|&a| [0u8, 0u8, 0u8, a]).collect()
            }
            Compression::L8 => {
                let unswiz = unswizzle_rect(source, width, height, 1);
                unswiz.iter().flat_map(|&l| [l, l, l, 255u8]).collect()
            }
            Compression::A8L8 => {
                let unswiz = unswizzle_rect(source, width, height, 2);
                unswiz
                    .chunks_exact(2)
                    .flat_map(|p| {
                        let l = p[0];
                        let a = p[1];
                        [l, l, l, a]
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
            Compression::DXT1 => decode_dxt(source, width, height, Format::BC1_UNORM),
            Compression::DXT2 => {
                decode_dxt(source, width, height, Format::BC2_UNORM_PREMULTIPLIED_ALPHA)
            }
            Compression::DXT3 => decode_dxt(source, width, height, Format::BC2_UNORM),
            Compression::DXT4 => {
                decode_dxt(source, width, height, Format::BC3_UNORM_PREMULTIPLIED_ALPHA)
            }
            Compression::DXT5 => decode_dxt(source, width, height, Format::BC3_UNORM),
            Compression::X8R8G8B8 => {
                let unswiz = unswizzle_rect(source, width, height, 4);
                unswiz
                    .chunks_exact(4)
                    .flat_map(|p| {
                        let b = p[0];
                        let g = p[1];
                        let r = p[2];
                        [r, g, b, 255u8]
                    })
                    .collect()
            }
            Compression::P8 { n_palettes: _ } => {
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
            // these seem to be unused in the actual assets, skipping
            Compression::V8U8
            | Compression::L6V5U5
            | Compression::UYVY
            | Compression::D24S8
            | Compression::D16
            | Compression::UNKNOWN => todo!("decode not supported for {self:?}"),
        }
    }
}

/// Decode raw DXT block data into RGBA8 pixels (flipped vertically).
fn decode_dxt(source: &[u8], width: usize, height: usize, format: Format) -> Vec<u8> {
    let four_cc = match format {
        Format::BC1_UNORM => FourCC::DXT1,
        Format::BC2_UNORM => FourCC::DXT3,
        Format::BC2_UNORM_PREMULTIPLIED_ALPHA => FourCC::DXT2,
        Format::BC3_UNORM => FourCC::DXT5,
        Format::BC3_UNORM_PREMULTIPLIED_ALPHA => FourCC::DXT4,
        _ => panic!("decode_dxt: unsupported format {format:?}"),
    };
    let header = Header::Dx9(Dx9Header::new_image(
        width as u32,
        height as u32,
        Dx9PixelFormat::FourCC(four_cc),
    ));
    let mut decoder = Decoder::from_header_with(Cursor::new(source), header, format)
        .expect("failed to create DXT decoder");
    let mut rgba = vec![0u8; width * height * 4];
    let view = ImageViewMut::new(
        &mut rgba,
        Size::new(width as u32, height as u32),
        ColorFormat::RGBA_U8,
    )
    .expect("failed to create image view");
    decoder
        .read_surface(view)
        .expect("failed to decode DXT surface");

    // ART stores DXT upside-down, so flip after decoding
    rgba.chunks_exact(width * 4)
        .rev()
        .flat_map(|r| r.iter().copied())
        .collect()
}
