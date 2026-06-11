mod common;

use crate::common::data::{PHILIPS, SMPTE};
use crate::common::util::{assert_pixels_close, image_from_png};
use ct3lib::data::{Compression, Image};

#[test]
fn to_png_produces_valid_png() {
    for (label, png_src, compression) in [
        ("R5G6B5/Philips", PHILIPS, Compression::R5G6B5),
        ("A8R8G8B8/SMPTE", SMPTE, Compression::A8R8G8B8),
        ("DXT1/SMPTE", SMPTE, Compression::DXT1),
    ] {
        let img = image_from_png(png_src, compression, 1);
        let png_bytes = img.to_png();
        assert_eq!(
            &png_bytes[..8],
            b"\x89PNG\r\n\x1a\n",
            "{label}: PNG signature invalid"
        );
        let decoded = image::load_from_memory(&png_bytes)
            .unwrap_or_else(|e| panic!("{label}: load_from_memory failed: {e}"))
            .into_rgba8();
        assert_eq!(
            decoded.width(),
            img.header.width as u32,
            "{label}: PNG width mismatch"
        );
        assert_eq!(
            decoded.height(),
            img.header.height as u32,
            "{label}: PNG height mismatch"
        );
    }
}

#[test]
fn png_encode_decode_roundtrip_lossless() {
    // A8R8G8B8 is lossless: PNG → Image → to_png → from_png_bytes → decode must be pixel-identical
    let img = image_from_png(PHILIPS, Compression::A8R8G8B8, 1);
    let original_rgba = img.decode();
    let png_bytes = img.to_png();
    let reimported =
        Image::from_png_bytes(&png_bytes, Compression::A8R8G8B8, 1).expect("from_png_bytes failed");
    let roundtrip_rgba = reimported.decode();
    assert_eq!(
        original_rgba, roundtrip_rgba,
        "A8R8G8B8 PNG roundtrip not pixel-identical"
    );
}

#[test]
fn png_encode_decode_roundtrip_lossy() {
    // For lossy formats, allow small per-channel tolerance
    for (label, compression, tolerance) in [
        ("R5G6B5", Compression::R5G6B5, 8u8),
        ("A4R4G4B4", Compression::A4R4G4B4, 16u8),
        ("DXT1", Compression::DXT1, 32u8),
        ("DXT3", Compression::DXT3, 32u8),
    ] {
        let img = image_from_png(PHILIPS, compression, 1);
        let original_rgba = img.decode();
        let png_bytes = img.to_png();
        let reimported = Image::from_png_bytes(&png_bytes, compression, 1)
            .unwrap_or_else(|e| panic!("{label}: from_png_bytes failed: {e}"));
        let roundtrip_rgba = reimported.decode();
        assert_pixels_close(&original_rgba, &roundtrip_rgba, tolerance, label);
    }
}
