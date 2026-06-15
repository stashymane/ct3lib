mod common;

use crate::common::data::{PHILIPS, SMPTE};
use crate::common::util::{image_from_png, roundtrip};
use ct3lib::art::compression::Compression;

#[test]
fn encode_decode_r5g6b5() {
    let img = image_from_png(PHILIPS, Compression::R5G6B5, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(orig.header.width, rt.header.width);
    assert_eq!(orig.header.height, rt.header.height);
    assert_eq!(rt.header.compression, Compression::R5G6B5);
    assert_eq!(orig.data, rt.data, "R5G6B5 data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_a4r4g4b4() {
    let img = image_from_png(PHILIPS, Compression::A4R4G4B4, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, Compression::A4R4G4B4);
    assert_eq!(orig.data, rt.data, "A4R4G4B4 data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_a1r5g5b5() {
    let img = image_from_png(PHILIPS, Compression::A1R5G5B5, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, Compression::A1R5G5B5);
    assert_eq!(orig.data, rt.data, "A1R5G5B5 data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_l8() {
    let img = image_from_png(PHILIPS, Compression::L8, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, Compression::L8);
    assert_eq!(orig.data, rt.data, "L8 data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_a8l8() {
    let img = image_from_png(PHILIPS, Compression::A8L8, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, Compression::A8L8);
    assert_eq!(orig.data, rt.data, "A8L8 data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_a8r8g8b8() {
    let img = image_from_png(PHILIPS, Compression::A8R8G8B8, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, Compression::A8R8G8B8);
    assert_eq!(orig.data, rt.data, "A8R8G8B8 data mismatch after roundtrip");
    // A8R8G8B8 is lossless: decode → re-encode → decode must be pixel-identical
    let rgba_orig = orig.decode();
    let rgba_rt = rt.decode();
    assert_eq!(rgba_orig, rgba_rt, "A8R8G8B8 pixel roundtrip not lossless");
}

#[test]
fn encode_decode_dxt1() {
    let img = image_from_png(SMPTE, Compression::DXT1, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, Compression::DXT1);
    assert_eq!(orig.data, rt.data, "DXT1 data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_dxt3() {
    let img = image_from_png(SMPTE, Compression::DXT3, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, Compression::DXT3);
    assert_eq!(orig.data, rt.data, "DXT3 data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_dxt2() {
    let img = image_from_png(SMPTE, Compression::DXT2, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, Compression::DXT2);
    assert_eq!(orig.data, rt.data, "DXT2 data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_dxt5() {
    let img = image_from_png(SMPTE, Compression::DXT5, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, Compression::DXT5);
    assert_eq!(orig.data, rt.data, "DXT5 data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_paletted() {
    // Use a small synthetic image so the palette fits in 256 entries
    let width: u16 = 16;
    let height: u16 = 16;
    // 4 distinct colours repeated
    let mut rgba_pixels = Vec::with_capacity(width as usize * height as usize * 4);
    let colours: [[u8; 4]; 4] = [
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [128, 128, 128, 255],
    ];
    for i in 0..(width as usize * height as usize) {
        rgba_pixels.extend_from_slice(&colours[i % 4]);
    }
    // Build a PNG from the raw pixels
    let png_bytes = {
        let img = image::RgbaImage::from_raw(width as u32, height as u32, rgba_pixels.clone())
            .expect("failed to create test image");
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("png encode");
        buf
    };

    let compression = Compression::P8 { n_palettes: 1 };
    let img = image_from_png(&png_bytes, compression, 1);
    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.compression, compression);
    assert_eq!(orig.data, rt.data, "Paletted data mismatch after roundtrip");
    let rgba = rt.decode();
    assert_eq!(rgba.len(), width as usize * height as usize * 4);
}

#[test]
fn compression_roundtrip_values() {
    let variants = [
        Compression::A1R5G5B5,
        Compression::A4R4G4B4,
        Compression::A8,
        Compression::L8,
        Compression::A8L8,
        Compression::R5G6B5,
        Compression::A8R8G8B8,
        Compression::X8R8G8B8,
        Compression::V8U8,
        Compression::L6V5U5,
        Compression::DXT1,
        Compression::DXT3,
        Compression::DXT5,
        Compression::DXT2,
        Compression::DXT4,
        Compression::P8 { n_palettes: 1 },
        Compression::P8 { n_palettes: 3 },
        Compression::UYVY,
        Compression::D24S8,
        Compression::UNKNOWN,
        Compression::D16,
    ];
    for &c in &variants {
        let v = c.to_u32();
        assert_eq!(
            Compression::from_u32(v),
            Some(c),
            "roundtrip failed for {c:?}"
        );
    }
}
