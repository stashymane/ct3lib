mod common;
use crate::common::data::{PHILIPS, SMPTE};
use crate::common::util::image_from_png;
use common::util::encode_to_vec;
use ct3lib::data::{Compression, Image, ImageHeader};
use ct3lib::{Art, ArtDecoder, ArtEncoder};
use std::io::Read;

#[test]
fn streaming_decoder_yields_correct_count() {
    // Build a 3-image ART in memory, then stream-decode it
    let images: Vec<Image> = [
        Compression::R5G6B5,
        Compression::DXT1,
        Compression::A8R8G8B8,
    ]
    .iter()
    .map(|&c| image_from_png(SMPTE, c, 1))
    .collect();
    let art = Art { images };
    let encoded = encode_to_vec(&art);

    let mut decoder = ArtDecoder::new(encoded.as_slice()).expect("failed to create decoder");
    assert_eq!(decoder.len(), 3);
    let mut count = 0;
    while let Some((header, mut data_reader)) = decoder.next_entry().expect("next_entry failed") {
        let mut buf = vec![0u8; header.total_data_size()];
        data_reader.read_exact(&mut buf).expect("read data failed");
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn streaming_encoder_produces_identical_output() {
    let images: Vec<Image> = [Compression::R5G6B5, Compression::A8R8G8B8]
        .iter()
        .map(|&c| image_from_png(PHILIPS, c, 1))
        .collect();
    let art = Art { images };
    let reference = encode_to_vec(&art);

    // Now encode the same images via the streaming API
    let entries: Vec<(ImageHeader, usize)> = art
        .images
        .iter()
        .map(|img| (img.header.clone(), img.data.len()))
        .collect();
    let mut buf = Vec::new();
    let mut encoder = ArtEncoder::new(&mut buf, entries).expect("encoder new failed");
    for img in &art.images {
        encoder
            .write_image(img.data.as_slice())
            .expect("write_image failed");
    }

    assert_eq!(
        buf, reference,
        "streaming encoder output differs from Art::encode"
    );
}
