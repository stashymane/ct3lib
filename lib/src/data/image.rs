use crate::data::compression::Compression;

/// Image metadata without the raw pixel data — used for streaming APIs.
#[derive(Debug, Clone)]
pub struct ImageHeader {
    pub width: u16,
    pub height: u16,
    pub compression: Compression,
    /// Number of mipmap levels (1 = base image only, >1 = mip chain).
    /// This is the low 16 bits of the raw compression u32 from the file.
    pub mip_count: u16,
}

impl ImageHeader {
    /// Total byte size of all mip levels as stored in the file.
    pub fn total_data_size(&self) -> usize {
        let mut size = 0;
        let mut w = self.width as usize;
        let mut h = self.height as usize;
        for _ in 0..self.mip_count {
            size += self.compression.base_mip_size(w, h);
            if w == 1 && h == 1 {
                break;
            }
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }
        size
    }

    /// The raw u32 compression field value to write into the file.
    pub fn comp_u32(&self) -> u32 {
        (self.compression.to_u32() & 0xffff_0000) | self.mip_count as u32
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    pub header: ImageHeader,
    /// Raw compressed/encoded image data as stored in the file
    pub data: Vec<u8>,
}

impl Image {
    pub fn width(&self) -> u16 { self.header.width }
    pub fn height(&self) -> u16 { self.header.height }
    pub fn compression(&self) -> Compression { self.header.compression }
    pub fn mip_count(&self) -> u16 { self.header.mip_count }
}

impl Image {
    /// Encode RGBA8 pixels (row-major, top-to-bottom) into the image's compression format,
    /// flipping vertically to match the upside-down storage convention.
    pub fn encode_rgba(&self, rgba: &[u8]) -> Vec<u8> {
        let w = self.header.width as usize;
        let h = self.header.height as usize;

        // Flip vertically (the format stores images upside-down)
        let flipped: Vec<u8> = rgba
            .chunks_exact(w * 4)
            .rev()
            .flat_map(|row| row.iter().copied())
            .collect();

        self.header.compression.encode(&flipped, w, h)
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
            // Flip vertically (format stores images upside-down)
            let flipped = imageops::flip_vertical(&current);
            out.extend(self.header.compression.encode(flipped.as_raw(), w as usize, h as usize));
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

    /// Decode the image data into RGBA8 pixels (row-major, top-to-bottom)
    pub fn decode(&self) -> Vec<u8> {
        let w = self.header.width as usize;
        let h = self.header.height as usize;
        // When mip_count > 1 the data contains a mip chain; slice off only the
        // base (mip 0) image before handing it to the decompressor.
        let base_size = self.header.compression.base_mip_size(w, h);
        let base_data = &self.data[..base_size.min(self.data.len())];
        let raw = self.header.compression.decode(base_data, w, h);

        // The image data is stored upside-down; flip vertically to produce the correct orientation.
        raw.chunks_exact(w * 4)
            .rev()
            .flat_map(|row| row.iter().copied())
            .collect()
    }
}
