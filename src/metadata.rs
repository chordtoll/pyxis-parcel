use serde::{Deserialize, Serialize};

/// Struct for parcel packaging metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ParcelMetadata {
    pub version: String,
    pub depends: Vec<String>,
}

impl ParcelMetadata {
    /// Creates a new empty metadata
    pub fn new() -> Self {
        Self{
            version: String::new(),
            depends: Vec::new(),
        }
    }
}