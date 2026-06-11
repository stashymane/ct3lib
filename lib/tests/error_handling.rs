mod common;
use ct3lib::{Art, DecodeError};

#[test]
fn decode_error_unexpected_eof() {
    assert!(matches!(Art::decode(&[][..]), Err(DecodeError::Io(_))));
    assert!(matches!(
        Art::decode(&[1, 0, 0, 0][..]),
        Err(DecodeError::Io(_))
    ));
}

#[test]
fn decode_error_invalid_magic() {
    let mut data = vec![0u8; 24];
    data[0] = 1; // count = 1
    data[4] = 8; // ptr[0] = 8
    // magic at offset 8 is all zeros → invalid
    assert!(matches!(
        Art::decode(data.as_slice()),
        Err(DecodeError::InvalidMagic { .. })
    ));
}

#[test]
fn decode_error_unknown_compression() {
    let magic = u32::from_le_bytes(*b"GXTX");
    let mut data = vec![0u8; 24];
    data[0] = 1; // count = 1
    data[4] = 8; // ptr[0] = 8
    data[8..12].copy_from_slice(&magic.to_le_bytes());
    data[8 + 4..8 + 6].copy_from_slice(&1u16.to_le_bytes()); // width=1
    data[8 + 6..8 + 8].copy_from_slice(&1u16.to_le_bytes()); // height=1
    let bad_comp: u32 = (99u32 << 16) | 1;
    data[8 + 12..8 + 16].copy_from_slice(&bad_comp.to_le_bytes());
    assert!(matches!(
        Art::decode(data.as_slice()),
        Err(DecodeError::UnknownCompression { .. })
    ));
}
