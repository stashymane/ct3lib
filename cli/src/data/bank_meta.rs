use ct3lib::data::Compression;
use ct3lib::Art;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct BankMetadata {
    pub name: String,
    pub metadata: BTreeMap<usize, ImageMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub compression: Compression,
    /// Number of mipmap levels stored in the ART file (1 = base image only).
    pub mip_count: u16,
}

impl BankMetadata {
    pub fn from(name: String, art: &Art) -> Self {
        let metadata = art
            .images
            .iter()
            .enumerate()
            .map(|(i, img)| {
                (
                    i,
                    ImageMetadata {
                        compression: img.header.compression,
                        mip_count: img.header.mip_count,
                    },
                )
            })
            .collect();

        Self { name, metadata }
    }

    pub fn get_filename(&self) -> String {
        format!("{}.art", self.name)
    }
}
