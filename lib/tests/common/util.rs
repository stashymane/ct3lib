use ct3lib::data::{Compression, Image};
use ct3lib::Art;

pub fn encode_to_vec(art: &Art) -> Vec<u8> {
    let mut buf = Vec::new();
    art.encode(&mut buf).expect("encode failed");
    buf
}

/// Encode a PNG byte slice into an `Image` with the given compression and mip_count.
pub fn image_from_png(png: &[u8], compression: Compression, mip_count: u16) -> Image {
    Image::from_png_bytes(png, compression, mip_count)
        .unwrap_or_else(|e| panic!("from_png_bytes failed for {compression:?}: {e}"))
}

/// Build a single-image `Art`, encode it to bytes, decode it back, and return
/// the decoded `Image`.
pub fn roundtrip(img: Image) -> (Image, Image) {
    let original = img.clone();
    let art = Art { images: vec![img] };
    let encoded = encode_to_vec(&art);
    let art2 = Art::decode(encoded.as_slice()).expect("decode after encode failed");
    assert_eq!(art2.images.len(), 1);
    (original, art2.images.into_iter().next().unwrap())
}

/// Assert that two RGBA pixel buffers are equal up to `tolerance` per channel.
pub fn assert_pixels_close(a: &[u8], b: &[u8], tolerance: u8, label: &str) {
    assert_eq!(a.len(), b.len(), "{label}: pixel buffer length mismatch");
    for (i, (&x, &y)) in a.iter().zip(b.iter()).enumerate() {
        let diff = (x as i16 - y as i16).unsigned_abs() as u8;
        assert!(
            diff <= tolerance,
            "{label}: byte {i}: original={x} roundtrip={y} diff={diff} exceeds tolerance {tolerance}"
        );
    }
}
