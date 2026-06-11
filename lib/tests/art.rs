mod common;

use crate::common::data::SMPTE;
use crate::common::util::{encode_to_vec, image_from_png};
use ct3lib::data::{Compression, Image};
use ct3lib::Art;

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

    let art = Art { images };
    let encoded = encode_to_vec(&art);
    let art2 = Art::decode(encoded.as_slice()).expect("multi-image decode failed");

    assert_eq!(art2.images.len(), compressions.len());
    for (i, (orig, rt)) in art.images.iter().zip(art2.images.iter()).enumerate() {
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
    let art = Art { images: vec![] };
    let encoded = encode_to_vec(&art);
    let art2 = Art::decode(encoded.as_slice()).expect("empty art decode failed");
    assert_eq!(art2.images.len(), 0);
}
