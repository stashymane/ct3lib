mod common;

use crate::common::data::{PHILIPS, SMPTE};
use crate::common::util::{assert_pixels_close, image_from_dds};
use ct3lib::art::compression::Compression;
use ct3lib::art::image::Image;

#[test]
fn to_dds_produces_valid_dds() {
    for (label, png_src, compression) in [
        ("R5G6B5/Philips", PHILIPS, Compression::R5G6B5),
        ("A8R8G8B8/SMPTE", SMPTE, Compression::A8R8G8B8),
        ("DXT1/SMPTE", SMPTE, Compression::DXT1),
    ] {
        let img = image_from_dds(png_src, compression, 1);
        let dds_bytes = img
            .to_dds()
            .unwrap_or_else(|| panic!("{label}: to_dds returned None"));
        assert_eq!(&dds_bytes[..4], b"DDS ", "{label}: DDS magic invalid");
        assert_eq!(
            img.header.width as u32,
            u32::from_le_bytes(dds_bytes[16..20].try_into().unwrap()),
            "{label}: DDS height field mismatch"
        );
        assert_eq!(
            img.header.height as u32,
            u32::from_le_bytes(dds_bytes[12..16].try_into().unwrap()),
            "{label}: DDS width field mismatch"
        );
    }
}

#[test]
fn dds_encode_decode_roundtrip_lossless() {
    // A8R8G8B8 is lossless: DDS → Image → to_dds → from_dds_bytes → decode must be pixel-identical
    let img = image_from_dds(PHILIPS, Compression::A8R8G8B8, 1);
    let original_rgba = img.decode();
    let dds_bytes = img.to_dds().expect("to_dds failed");
    let reimported =
        Image::from_dds_bytes(&dds_bytes, Compression::A8R8G8B8, 1).expect("from_dds_bytes failed");
    let roundtrip_rgba = reimported.decode();
    assert_eq!(
        original_rgba, roundtrip_rgba,
        "A8R8G8B8 DDS roundtrip not pixel-identical"
    );
}

#[test]
fn dds_encode_decode_roundtrip_lossy() {
    // For lossy formats, allow small per-channel tolerance
    for (label, compression, tolerance) in [
        ("R5G6B5", Compression::R5G6B5, 8u8),
        ("A4R4G4B4", Compression::A4R4G4B4, 16u8),
        ("DXT1", Compression::DXT1, 32u8),
        ("DXT3", Compression::DXT3, 32u8),
    ] {
        let img = image_from_dds(PHILIPS, compression, 1);
        let original_rgba = img.decode();
        let dds_bytes = img
            .to_dds()
            .unwrap_or_else(|| panic!("{label}: to_dds returned None"));
        let reimported = Image::from_dds_bytes(&dds_bytes, compression, 1)
            .unwrap_or_else(|e| panic!("{label}: from_dds_bytes failed: {e}"));
        let roundtrip_rgba = reimported.decode();
        assert_pixels_close(&original_rgba, &roundtrip_rgba, tolerance, label);
    }
}

#[test]
fn compression_from_dds_bytes_roundtrip() {
    for (label, compression) in [
        ("R5G6B5", Compression::R5G6B5),
        ("A8R8G8B8", Compression::A8R8G8B8),
        ("A1R5G5B5", Compression::A1R5G5B5),
        ("A4R4G4B4", Compression::A4R4G4B4),
        ("L8", Compression::L8),
        ("A8", Compression::A8),
        ("A8L8", Compression::A8L8),
        ("X8R8G8B8", Compression::X8R8G8B8),
        ("DXT1", Compression::DXT1),
        ("DXT3", Compression::DXT3),
        ("DXT5", Compression::DXT5),
    ] {
        let img = image_from_dds(PHILIPS, compression, 1);
        let dds_bytes = img
            .to_dds()
            .unwrap_or_else(|| panic!("{label}: to_dds returned None"));
        let detected = Image::compression_from_dds_bytes(&dds_bytes)
            .unwrap_or_else(|| panic!("{label}: compression_from_dds_bytes returned None"));
        assert_eq!(
            detected, compression,
            "{label}: detected compression mismatch"
        );
    }
}
