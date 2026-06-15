use crate::art::image_header::ImageHeader;
use crate::art::MAGIC;
use std::io;
use std::io::{Read, Write};

/// Streaming ART encoder. Requires all [`ImageHeader`]s and their exact data
/// sizes upfront so the pointer table can be written before any image data.
pub struct ArtEncoder<W: Write> {
    writer: W,
    entries: Vec<(ImageHeader, usize)>,
    written: usize,
}

impl<W: Write> ArtEncoder<W> {
    /// Create a new encoder and immediately write the file header (count +
    /// pointer table). Each entry is `(header, data_size)` where `data_size`
    /// is the exact number of bytes that will be passed to [`write_image`].
    pub fn new(mut writer: W, entries: Vec<(ImageHeader, usize)>) -> io::Result<Self> {
        let count = entries.len();
        let header_size = 4 + count * 4;

        // Compute pointer for each image
        let mut offset = header_size;
        let ptrs: Vec<u32> = entries
            .iter()
            .map(|(_, size)| {
                let ptr = offset as u32;
                offset += 16 + size;
                ptr
            })
            .collect();

        writer.write_all(&(count as u32).to_le_bytes())?;
        for ptr in &ptrs {
            writer.write_all(&ptr.to_le_bytes())?;
        }

        Ok(Self {
            writer,
            entries,
            written: 0,
        })
    }

    /// Write the next image. The provided reader must yield exactly the
    /// `data_size` bytes declared for this entry in [`ArtEncoder::new`].
    pub fn write_image<R: Read>(&mut self, mut data: R) -> io::Result<()> {
        let (h, size) = &self.entries[self.written];
        self.writer.write_all(&MAGIC.to_le_bytes())?;
        self.writer.write_all(&h.width.to_le_bytes())?;
        self.writer.write_all(&h.height.to_le_bytes())?;
        self.writer.write_all(&(*size as u32).to_le_bytes())?;
        self.writer.write_all(&h.comp_u32().to_le_bytes())?;
        io::copy(&mut data, &mut self.writer)?;
        self.written += 1;
        Ok(())
    }
}
