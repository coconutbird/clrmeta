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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_u8() {
        let mut writer = Writer::new();
        writer.write_u8(0x42);
        writer.write_u8(0x43);
        assert_eq!(writer.as_slice(), &[0x42, 0x43]);
    }

    #[test]
    fn test_write_u16() {
        let mut writer = Writer::new();
        writer.write_u16(0x0201);
        assert_eq!(writer.as_slice(), &[0x01, 0x02]);
    }

    #[test]
    fn test_write_u32() {
        let mut writer = Writer::new();
        writer.write_u32(0x04030201);
        assert_eq!(writer.as_slice(), &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_write_null_str() {
        let mut writer = Writer::new();
        writer.write_null_str("Hello");
        assert_eq!(writer.as_slice(), b"Hello\0");
    }

    #[test]
    fn test_write_compressed_uint_1byte() {
        let mut writer = Writer::new();
        writer.write_compressed_uint(0);
        writer.write_compressed_uint(127);
        assert_eq!(writer.as_slice(), &[0x00, 0x7F]);
    }

    #[test]
    fn test_write_compressed_uint_2bytes() {
        let mut writer = Writer::new();
        writer.write_compressed_uint(128);
        assert_eq!(writer.as_slice(), &[0x80, 0x80]);

        let mut writer2 = Writer::new();
        writer2.write_compressed_uint(16383);
        assert_eq!(writer2.as_slice(), &[0xBF, 0xFF]);
    }

    #[test]
    fn test_write_compressed_uint_4bytes() {
        let mut writer = Writer::new();
        writer.write_compressed_uint(16384);
        assert_eq!(writer.as_slice(), &[0xC0, 0x00, 0x40, 0x00]);
    }

    #[test]
    fn test_align() {
        let mut writer = Writer::new();
        writer.write_u8(0x42);
        writer.align(4);
        assert_eq!(writer.len(), 4);
        assert_eq!(writer.as_slice(), &[0x42, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_write_index() {
        let mut writer = Writer::new();
        writer.write_index(0x0201, false);
        writer.write_index(0x06050403, true);
        assert_eq!(writer.as_slice(), &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }

    #[test]
    fn test_reserve_and_patch() {
        let mut writer = Writer::new();
        let offset = writer.reserve(4);
        writer.write_u8(0xFF);
        writer.patch_u32(offset, 0x04030201);
        assert_eq!(writer.as_slice(), &[0x01, 0x02, 0x03, 0x04, 0xFF]);
    }
}
