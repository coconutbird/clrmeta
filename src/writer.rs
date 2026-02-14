//! Binary writer for serializing metadata structures.

/// A binary writer for producing little-endian data.
#[derive(Debug, Clone, Default)]
pub struct Writer {
    data: Vec<u8>,
}

impl Writer {
    /// Create a new empty writer.
    #[must_use]
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a new writer with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Get the current length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the writer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the written data.
    #[must_use]
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    /// Get a reference to the written data.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Write a single byte.
    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    /// Write a little-endian u16.
    pub fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Write a little-endian u32.
    pub fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Write a little-endian u64.
    pub fn write_u64(&mut self, value: u64) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Write a slice of bytes.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Write a null-terminated string.
    pub fn write_null_str(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(0);
    }

    /// Write padding to align to a boundary.
    pub fn align(&mut self, alignment: usize) {
        let remainder = self.data.len() % alignment;
        if remainder != 0 {
            let padding = alignment - remainder;
            self.data.resize(self.data.len() + padding, 0);
        }
    }

    /// Write a 2 or 4 byte index based on size flag.
    pub fn write_index(&mut self, value: u32, wide: bool) {
        if wide {
            self.write_u32(value);
        } else {
            self.write_u16(value as u16);
        }
    }

    /// Write a compressed unsigned integer (ECMA-335 II.23.2).
    pub fn write_compressed_uint(&mut self, value: u32) {
        if value < 0x80 {
            // 1 byte: 0xxxxxxx
            self.write_u8(value as u8);
        } else if value < 0x4000 {
            // 2 bytes: 10xxxxxx xxxxxxxx
            self.write_u8((0x80 | (value >> 8)) as u8);
            self.write_u8(value as u8);
        } else {
            // 4 bytes: 110xxxxx xxxxxxxx xxxxxxxx xxxxxxxx
            self.write_u8((0xC0 | (value >> 24)) as u8);
            self.write_u8((value >> 16) as u8);
            self.write_u8((value >> 8) as u8);
            self.write_u8(value as u8);
        }
    }

    /// Reserve space and return the offset for later patching.
    pub fn reserve(&mut self, len: usize) -> usize {
        let offset = self.data.len();
        self.data.resize(offset + len, 0);
        offset
    }

    /// Patch a u32 value at a specific offset.
    pub fn patch_u32(&mut self, offset: usize, value: u32) {
        let bytes = value.to_le_bytes();
        self.data[offset..offset + 4].copy_from_slice(&bytes);
    }

    /// Patch a u16 value at a specific offset.
    pub fn patch_u16(&mut self, offset: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.data[offset..offset + 2].copy_from_slice(&bytes);
    }
}

