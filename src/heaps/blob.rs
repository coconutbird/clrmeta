//! #Blob heap - length-prefixed binary data.

use crate::error::{Error, Result};
use crate::reader::Reader;
use crate::writer::Writer;

/// The #Blob heap containing length-prefixed binary blobs.
#[derive(Debug, Clone, Default)]
pub struct BlobHeap {
    /// Raw heap data.
    data: Vec<u8>,
}

impl BlobHeap {
    /// Create a new empty blob heap.
    #[must_use]
    pub fn new() -> Self {
        // Heap always starts with a null byte (empty blob at index 0)
        Self { data: vec![0] }
    }

    /// Parse the blob heap from raw bytes.
    #[must_use]
    pub fn parse(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Get a blob at the given offset.
    pub fn get(&self, offset: u32) -> Result<&[u8]> {
        let offset = offset as usize;
        if offset >= self.data.len() {
            return Err(Error::InvalidBlob(offset));
        }

        let mut reader = Reader::new(&self.data[offset..]);
        let len = reader.read_compressed_uint()? as usize;

        let header_size = reader.position();
        let blob_start = offset + header_size;
        let blob_end = blob_start + len;

        if blob_end > self.data.len() {
            return Err(Error::InvalidBlob(offset));
        }

        Ok(&self.data[blob_start..blob_end])
    }

    /// Add a blob to the heap and return its offset.
    pub fn add(&mut self, blob: &[u8]) -> u32 {
        let offset = self.data.len() as u32;

        // Write compressed length
        let mut writer = Writer::new();
        writer.write_compressed_uint(blob.len() as u32);
        self.data.extend_from_slice(writer.as_slice());

        // Write blob data
        self.data.extend_from_slice(blob);

        offset
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

    /// Check if the heap uses 4-byte indices (size > 65535).
    #[must_use]
    pub fn uses_wide_indices(&self) -> bool {
        self.data.len() > 0xFFFF
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

    /// Iterate over all blobs in the heap with their offsets.
    pub fn iter(&self) -> BlobIter<'_> {
        BlobIter {
            heap: self,
            offset: 0,
        }
    }
}

/// Iterator over blobs in the heap.
pub struct BlobIter<'a> {
    heap: &'a BlobHeap,
    offset: usize,
}

impl<'a> Iterator for BlobIter<'a> {
    type Item = (u32, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.heap.data.len() {
            return None;
        }

        let start = self.offset;
        let mut reader = Reader::new(&self.heap.data[self.offset..]);
        let len = reader.read_compressed_uint().ok()? as usize;
        let header_size = reader.position();

        self.offset += header_size + len;

        if self.offset > self.heap.data.len() {
            return None;
        }

        Some((
            start as u32,
            &self.heap.data[start + header_size..start + header_size + len],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_heap_has_empty_blob() {
        let heap = BlobHeap::new();
        assert_eq!(heap.get(0).unwrap(), &[] as &[u8]);
    }

    #[test]
    fn test_add_and_get_blob() {
        let mut heap = BlobHeap::new();
        let offset = heap.add(&[0x01, 0x02, 0x03]);
        assert_eq!(heap.get(offset).unwrap(), &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_parse_heap() {
        // Format: compressed length (1 byte for small) + data
        // Empty blob at 0, then blob [0xAB, 0xCD] at offset 1
        let data = [0x00, 0x02, 0xAB, 0xCD];
        let heap = BlobHeap::parse(&data);
        assert_eq!(heap.get(0).unwrap(), &[] as &[u8]);
        assert_eq!(heap.get(1).unwrap(), &[0xAB, 0xCD]);
    }

    #[test]
    fn test_write_heap() {
        let mut heap = BlobHeap::new();
        heap.add(&[0x42, 0x43]);
        let data = heap.write();
        // Empty blob (0x00), then length (0x02), then data
        assert_eq!(data, vec![0x00, 0x02, 0x42, 0x43]);
    }

    #[test]
    fn test_iter() {
        let data = [0x00, 0x02, 0xAB, 0xCD];
        let heap = BlobHeap::parse(&data);
        let blobs: Vec<_> = heap.iter().collect();
        assert_eq!(blobs.len(), 2);
        assert_eq!(blobs[0].0, 0);
        assert_eq!(blobs[0].1, &[] as &[u8]);
        assert_eq!(blobs[1].0, 1);
        assert_eq!(blobs[1].1, &[0xABu8, 0xCDu8]);
    }
}
