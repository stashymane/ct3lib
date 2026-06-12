use ct3lib::data::{Compression, ImageHeader};
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
    pub fn from_headers(name: String, headers: impl IntoIterator<Item = ImageHeader>) -> Self {
        let metadata = headers
            .into_iter()
            .enumerate()
            .map(|(i, h)| {
                (
                    i,
                    ImageMetadata {
                        compression: h.compression,
                        mip_count: h.mip_count,
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
