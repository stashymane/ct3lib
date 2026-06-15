mod common;

use crate::common::data::{PHILIPS, SMPTE};
use crate::common::util::{image_from_png, roundtrip};
use ct3lib::art::compression::Compression;

#[test]
fn encode_decode_mip_chain_r5g6b5() {
    // Use a 256×256 crop-equivalent: from_png_bytes uses the full image size,
    // mip chain goes down to 1×1.
    let mip_count = 4u16;
    let img = image_from_png(SMPTE, Compression::R5G6B5, mip_count);
    assert_eq!(img.header.mip_count, mip_count);
    let expected_size = img.header.total_data_size();
    assert_eq!(
        img.data.len(),
        expected_size,
        "mip chain data size mismatch"
    );

    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.mip_count, mip_count, "mip_count not preserved");
    assert_eq!(
        orig.data, rt.data,
        "mip chain data mismatch after roundtrip"
    );

    // Base level decodes to correct size
    let rgba = rt.decode();
    assert_eq!(
        rgba.len(),
        rt.header.width as usize * rt.header.height as usize * 4
    );
}

#[test]
fn encode_decode_mip_chain_dxt1() {
    let mip_count = 3u16;
    let img = image_from_png(SMPTE, Compression::DXT1, mip_count);
    assert_eq!(img.header.mip_count, mip_count);
    let expected_size = img.header.total_data_size();
    assert_eq!(
        img.data.len(),
        expected_size,
        "DXT1 mip chain data size mismatch"
    );

    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.mip_count, mip_count, "mip_count not preserved");
    assert_eq!(
        orig.data, rt.data,
        "DXT1 mip chain data mismatch after roundtrip"
    );
}

#[test]
fn encode_decode_mip_chain_a8r8g8b8() {
    let mip_count = 5u16;
    let img = image_from_png(PHILIPS, Compression::A8R8G8B8, mip_count);
    assert_eq!(img.header.mip_count, mip_count);
    let expected_size = img.header.total_data_size();
    assert_eq!(
        img.data.len(),
        expected_size,
        "A8R8G8B8 mip chain data size mismatch"
    );

    let (orig, rt) = roundtrip(img);
    assert_eq!(rt.header.mip_count, mip_count, "mip_count not preserved");
    assert_eq!(
        orig.data, rt.data,
        "A8R8G8B8 mip chain data mismatch after roundtrip"
    );
}
