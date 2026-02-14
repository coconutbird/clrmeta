//! #GUID heap - 16-byte GUIDs with 1-based indexing.

use crate::error::{Error, Result};
use crate::writer::Writer;

/// A GUID (16 bytes).
pub type Guid = [u8; 16];

/// The #GUID heap containing GUIDs (16-byte entries, 1-based indexing).
#[derive(Debug, Clone, Default)]
pub struct GuidHeap {
    /// Raw heap data (multiple of 16 bytes).
    data: Vec<u8>,
}

impl GuidHeap {
    /// Create a new empty GUID heap.
    #[must_use]
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Parse the GUID heap from raw bytes.
    #[must_use]
    pub fn parse(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Get a GUID by 1-based index.
    pub fn get(&self, index: u32) -> Result<Guid> {
        if index == 0 {
            // Index 0 means "null GUID"
            return Ok([0u8; 16]);
        }

        let offset = ((index - 1) as usize) * 16;
        if offset + 16 > self.data.len() {
            return Err(Error::InvalidGuidIndex(index));
        }

        let mut guid = [0u8; 16];
        guid.copy_from_slice(&self.data[offset..offset + 16]);
        Ok(guid)
    }

    /// Add a GUID to the heap and return its 1-based index.
    pub fn add(&mut self, guid: &Guid) -> u32 {
        let index = (self.data.len() / 16) + 1;
        self.data.extend_from_slice(guid);
        index as u32
    }

    /// Get the number of GUIDs in the heap.
    #[must_use]
    pub fn count(&self) -> usize {
        self.data.len() / 16
    }

    /// Get the raw heap data.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the size of the heap.
    #[must_use]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if the heap uses 4-byte indices (count > 65535).
    #[must_use]
    pub fn uses_wide_indices(&self) -> bool {
        self.count() > 0xFFFF
    }

    /// Write the heap to a writer.
    pub fn write_to(&self, writer: &mut Writer) {
        writer.write_bytes(&self.data);
    }

    /// Write the heap to bytes.
    #[must_use]
    pub fn write(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Iterate over all GUIDs in the heap with their 1-based indices.
    pub fn iter(&self) -> impl Iterator<Item = (u32, Guid)> + '_ {
        self.data.chunks_exact(16).enumerate().map(|(i, chunk)| {
            let mut guid = [0u8; 16];
            guid.copy_from_slice(chunk);
            ((i + 1) as u32, guid)
        })
    }
}

/// Format a GUID as a string (e.g., "550e8400-e29b-41d4-a716-446655440000").
#[must_use]
pub fn format_guid(guid: &Guid) -> String {
    // GUID layout: Data1 (4 bytes LE), Data2 (2 bytes LE), Data3 (2 bytes LE), Data4 (8 bytes)
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        guid[3], guid[2], guid[1], guid[0], // Data1 (LE)
        guid[5], guid[4],                   // Data2 (LE)
        guid[7], guid[6],                   // Data3 (LE)
        guid[8], guid[9],                   // Data4[0..2]
        guid[10], guid[11], guid[12], guid[13], guid[14], guid[15] // Data4[2..8]
    )
}

