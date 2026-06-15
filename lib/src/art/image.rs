use crate::art::compression::Compression;
use crate::art::image_header::ImageHeader;

#[derive(Debug, Clone)]
pub struct Image {
    pub header: ImageHeader,
    /// Raw compressed/encoded image data as stored in the file
    pub data: Vec<u8>,
}

impl Image {
    pub fn width(&self) -> u16 {
        self.header.width
    }
    pub fn height(&self) -> u16 {
        self.header.height
    }
    pub fn compression(&self) -> Compression {
        self.header.compression
    }
    pub fn mip_count(&self) -> u16 {
        self.header.mip_count
    }
}

impl Image {
    pub fn encode_rgba(&self, rgba: &[u8]) -> Vec<u8> {
        let w = self.header.width as usize;
        let h = self.header.height as usize;

        self.header.compression.encode(rgba, w, h)
    }

    /// Build the full mip chain data for this image given RGBA8 pixels of the base level.
    /// The returned `Vec<u8>` is the concatenation of all `mip_count` compressed mip levels.
    pub fn build_mip_chain(&self, base_rgba: &[u8]) -> Vec<u8> {
        use image::imageops;

        let mut out = Vec::new();
        let mut w = self.header.width as u32;
        let mut h = self.header.height as u32;
        let mut current = image::RgbaImage::from_raw(w, h, base_rgba.to_vec())
            .expect("build_mip_chain: invalid base image dimensions");

        for _ in 0..self.header.mip_count {
            out.extend(
                self.header
                    .compression
                    .encode(current.as_raw(), w as usize, h as usize),
            );
            if w == 1 && h == 1 {
                break;
            }
            let new_w = (w / 2).max(1);
            let new_h = (h / 2).max(1);
            current = imageops::resize(&current, new_w, new_h, imageops::FilterType::Triangle);
            w = new_w;
            h = new_h;
        }
        out
    }

    /// Decode the image data into RGBA8 pixels
    pub fn decode(&self) -> Vec<u8> {
        let w = self.header.width as usize;
        let h = self.header.height as usize;

        let base_size = self.header.compression.base_mip_size(w, h);
        let base_data = &self.data[..base_size.min(self.data.len())];

        self.header.compression.decode(base_data, w, h)
    }
}
