use ct3lib::data::{Compression, Image};
use ct3lib::Art;

pub fn encode_to_vec(images: &[Image]) -> Vec<u8> {
    let mut buf = Vec::new();
    Art::encode(&mut buf, images).expect("encode failed");
    buf
}

/// Encode a PNG byte slice into an `Image` with the given compression and mip_count.
pub fn image_from_png(png: &[u8], compression: Compression, mip_count: u16) -> Image {
    Image::from_png_bytes(png, compression, mip_count)
        .unwrap_or_else(|e| panic!("from_png_bytes failed for {compression:?}: {e}"))
}

/// Build a single-image ART, encode it to bytes, decode it back, and return
/// the decoded `Image`.
pub fn roundtrip(img: Image) -> (Image, Image) {
    let original = img.clone();
    let encoded = encode_to_vec(&[img]);
    let mut entries: Vec<Image> = Art::decode(encoded.as_slice())
        .expect("decode after encode failed")
        .into_iter()
        .map(|e| {
            let e = e.expect("entry error");
            Image {
                header: e.header,
                data: e.data,
            }
        })
        .collect();
    assert_eq!(entries.len(), 1);
    (original, entries.remove(0))
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
