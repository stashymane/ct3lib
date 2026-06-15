use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(usize)]
pub enum Compression {
    A1R5G5B5 = 0,
    A4R4G4B4 = 1,
    A8 = 2,
    L8 = 3,
    A8L8 = 4,
    R5G6B5 = 5,
    A8R8G8B8 = 6,
    X8R8G8B8 = 7,
    V8U8 = 8,
    L6V5U5 = 9,
    DXT1 = 10,
    DXT3 = 11,
    DXT5 = 12,
    DXT2 = 13,
    DXT4 = 14,
    P8 { n_palettes: u8 } = 15,
    UYVY = 16,
    D24S8 = 17,
    UNKNOWN = 18,
    D16 = 19,
}

impl Compression {
    pub fn from_u32(value: u32) -> Option<Self> {
        let hi = (value >> 16) as u16;
        let base_type = (hi & 0xff) as u8;
        let upper = (hi >> 8) as u8;
        // Paletted format: lower byte of hi word == 0x0f, upper byte == palette count
        if base_type == 0x0f {
            return Some(Compression::P8 { n_palettes: upper });
        }
        match hi {
            0 => Some(Compression::A1R5G5B5),
            1 => Some(Compression::A4R4G4B4),
            2 => Some(Compression::A8),
            3 => Some(Compression::L8),
            4 => Some(Compression::A8L8),
            5 => Some(Compression::R5G6B5),
            6 => Some(Compression::A8R8G8B8),
            7 => Some(Compression::X8R8G8B8),
            8 => Some(Compression::V8U8),
            9 => Some(Compression::L6V5U5),
            10 => Some(Compression::DXT1),
            11 => Some(Compression::DXT3),
            12 => Some(Compression::DXT5),
            13 => Some(Compression::DXT2),
            14 => Some(Compression::DXT4),
            16 => Some(Compression::UYVY),
            17 => Some(Compression::D24S8),
            18 => Some(Compression::UNKNOWN),
            19 => Some(Compression::D16),
            _ => None,
        }
    }

    pub fn to_u32(self) -> u32 {
        let hi: u32 = match self {
            Compression::A1R5G5B5 => 0,
            Compression::A4R4G4B4 => 1,
            Compression::A8 => 2,
            Compression::L8 => 3,
            Compression::A8L8 => 4,
            Compression::R5G6B5 => 5,
            Compression::A8R8G8B8 => 6,
            Compression::X8R8G8B8 => 7,
            Compression::V8U8 => 8,
            Compression::L6V5U5 => 9,
            Compression::DXT1 => 10,
            Compression::DXT3 => 11,
            Compression::DXT5 => 12,
            Compression::DXT2 => 13,
            Compression::DXT4 => 14,
            Compression::P8 { n_palettes } => ((n_palettes as u32) << 8) | 0x0f,
            Compression::UYVY => 16,
            Compression::D24S8 => 17,
            Compression::UNKNOWN => 18,
            Compression::D16 => 19,
        };
        (hi << 16) | 1
    }

    /// Returns the byte size of the base (mip 0) image only, ignoring any mip chain.
    pub fn base_mip_size(self, width: usize, height: usize) -> usize {
        match self {
            Compression::A8 | Compression::L8 => width * height,
            Compression::A4R4G4B4
            | Compression::A8L8
            | Compression::A1R5G5B5
            | Compression::R5G6B5
            | Compression::V8U8
            | Compression::L6V5U5 => width * height * 2,
            Compression::A8R8G8B8 | Compression::X8R8G8B8 | Compression::D24S8 => {
                width * height * 4
            }
            Compression::DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Compression::DXT2 | Compression::DXT3 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Compression::DXT4 | Compression::DXT5 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Compression::P8 { n_palettes } => width * height + n_palettes as usize * 256 * 4,
            Compression::UYVY => width * height * 2,
            Compression::D16 => width * height * 2,
            Compression::UNKNOWN => 0,
        }
    }
}
