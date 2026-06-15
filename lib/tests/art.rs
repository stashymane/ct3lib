mod common;

use crate::common::data::SMPTE;
use crate::common::util::{encode_to_vec, image_from_png};
use ct3lib::art::compression::Compression;
use ct3lib::art::image::Image;
use ct3lib::art::Art;

#[test]
fn multi_image_art_encode_decode() {
    let compressions = [
        Compression::R5G6B5,
        Compression::A8R8G8B8,
        Compression::DXT1,
        Compression::DXT3,
    ];
    let images: Vec<Image> = compressions
        .iter()
        .map(|&c| image_from_png(SMPTE, c, 1))
        .collect();

    let encoded = encode_to_vec(&images);
    let decoded: Vec<Image> = Art::decode(encoded.as_slice())
        .expect("multi-image decode failed")
        .into_iter()
        .map(|e| {
            let e = e.expect("entry error");
            Image {
                header: e.header,
                data: e.data,
            }
        })
        .collect();

    assert_eq!(decoded.len(), compressions.len());
    for (i, (orig, rt)) in images.iter().zip(decoded.iter()).enumerate() {
        assert_eq!(
            orig.header.width, rt.header.width,
            "image {i} width mismatch"
        );
        assert_eq!(
            orig.header.height, rt.header.height,
            "image {i} height mismatch"
        );
        assert_eq!(
            orig.header.compression, rt.header.compression,
            "image {i} compression mismatch"
        );
        assert_eq!(orig.data, rt.data, "image {i} data mismatch");
    }
}

#[test]
fn empty_art_encode_decode() {
    let encoded = encode_to_vec(&[]);
    let decoded: Vec<_> = Art::decode(encoded.as_slice())
        .expect("empty art decode failed")
        .into_iter()
        .collect();
    assert_eq!(decoded.len(), 0);
}
