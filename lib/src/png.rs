use crate::data::{Compression, Image, ImageHeader};
use image::ImageReader;
use std::io;
use std::io::Cursor;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PngError {
    #[error("An IO error has occurred")]
    IoError(#[from] io::Error),
    #[error("An error occurred decoding image data")]
    ImageError(#[from] image::ImageError),
}

impl Image {
    /// Encode the decoded RGBA8 pixels as a PNG file in memory
    pub fn to_png(&self) -> Vec<u8> {
        let rgba = self.decode();
        let img =
            image::RgbaImage::from_raw(self.header.width as u32, self.header.height as u32, rgba)
                .expect("failed to create image buffer");
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("png encode");
        buf
    }

    pub fn from_png_bytes(
        data: &[u8],
        compression: Compression,
        mip_count: u16,
    ) -> Result<Self, PngError> {
        let img = ImageReader::new(Cursor::new(data))
            .with_guessed_format()?
            .decode()?;
        Ok(Self::from_dynamic_image(img, compression, mip_count))
    }

    pub fn from_png(
        path: &Path,
        compression: Compression,
        mip_count: u16,
    ) -> Result<Self, PngError> {
        let img = ImageReader::open(path)?.decode()?;
        Ok(Self::from_dynamic_image(img, compression, mip_count))
    }

    fn from_dynamic_image(
        img: image::DynamicImage,
        compression: Compression,
        mip_count: u16,
    ) -> Self {
        let rgba = img.into_rgba8();
        let width = rgba.width() as u16;
        let height = rgba.height() as u16;
        let raw: Vec<u8> = rgba.into_raw();

        let mut image = Self {
            header: ImageHeader {
                width,
                height,
                size: 0,
                compression,
                mip_count,
            },
            data: Vec::new(),
        };

        let data = if mip_count > 1 {
            image.build_mip_chain(&raw)
        } else {
            image.encode_rgba(&raw)
        };

        image.header.size = image.header.total_data_size() as u32;

        Self { data, ..image }
    }
}
