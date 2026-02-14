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

