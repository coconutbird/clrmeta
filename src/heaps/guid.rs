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
    pub fn iter(&self) -> GuidIter<'_> {
        GuidIter {
            chunks: self.data.chunks_exact(16),
            index: 1,
        }
    }
}

/// Iterator over GUIDs in the heap.
pub struct GuidIter<'a> {
    chunks: std::slice::ChunksExact<'a, u8>,
    index: u32,
}

impl Iterator for GuidIter<'_> {
    type Item = (u32, Guid);

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next().map(|chunk| {
            let mut guid = [0u8; 16];
            guid.copy_from_slice(chunk);
            let idx = self.index;
            self.index += 1;
            (idx, guid)
        })
    }
}

impl<'a> IntoIterator for &'a GuidHeap {
    type Item = (u32, Guid);
    type IntoIter = GuidIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Format a GUID as a string (e.g., "550e8400-e29b-41d4-a716-446655440000").
#[must_use]
pub fn format_guid(guid: &Guid) -> String {
    // GUID layout: Data1 (4 bytes LE), Data2 (2 bytes LE), Data3 (2 bytes LE), Data4 (8 bytes)
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        guid[3],
        guid[2],
        guid[1],
        guid[0], // Data1 (LE)
        guid[5],
        guid[4], // Data2 (LE)
        guid[7],
        guid[6], // Data3 (LE)
        guid[8],
        guid[9], // Data4[0..2]
        guid[10],
        guid[11],
        guid[12],
        guid[13],
        guid[14],
        guid[15] // Data4[2..8]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_heap_is_empty() {
        let heap = GuidHeap::new();
        assert_eq!(heap.count(), 0);
    }

    #[test]
    fn test_null_guid_index() {
        let heap = GuidHeap::new();
        // Index 0 returns null GUID
        let guid = heap.get(0).unwrap();
        assert_eq!(guid, [0u8; 16]);
    }

    #[test]
    fn test_add_and_get_guid() {
        let mut heap = GuidHeap::new();
        let guid: Guid = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let index = heap.add(&guid);
        assert_eq!(index, 1); // 1-based indexing
        assert_eq!(heap.get(index).unwrap(), guid);
    }

    #[test]
    fn test_parse_heap() {
        let data: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // GUID 1
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, // GUID 2
        ];
        let heap = GuidHeap::parse(&data);
        assert_eq!(heap.count(), 2);
        assert_eq!(
            heap.get(1).unwrap(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
        assert_eq!(
            heap.get(2).unwrap(),
            [
                17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
            ]
        );
    }

    #[test]
    fn test_format_guid() {
        // Standard test GUID: {550e8400-e29b-41d4-a716-446655440000}
        // In memory (little-endian for first 3 parts):
        let guid: Guid = [
            0x00, 0x84, 0x0e, 0x55, // Data1: 550e8400 (LE)
            0x9b, 0xe2, // Data2: e29b (LE)
            0xd4, 0x41, // Data3: 41d4 (LE)
            0xa7, 0x16, // Data4[0..2]
            0x44, 0x66, 0x55, 0x44, 0x00, 0x00, // Data4[2..8]
        ];
        let formatted = format_guid(&guid);
        assert_eq!(formatted, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_iter() {
        let data: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let heap = GuidHeap::parse(&data);
        let guids: Vec<_> = heap.iter().collect();
        assert_eq!(guids.len(), 2);
        assert_eq!(guids[0].0, 1); // 1-based index
        assert_eq!(guids[1].0, 2);
    }
}
