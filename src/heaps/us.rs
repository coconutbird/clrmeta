//! #US (User Strings) heap - length-prefixed UTF-16LE strings.

use crate::error::{Error, Result};
use crate::reader::Reader;
use crate::writer::Writer;

/// The #US heap containing user strings (UTF-16LE with length prefix).
#[derive(Debug, Clone, Default)]
pub struct UserStringsHeap {
    /// Raw heap data.
    data: Vec<u8>,
}

impl UserStringsHeap {
    /// Create a new empty user strings heap.
    #[must_use]
    pub fn new() -> Self {
        // Heap always starts with a null byte
        Self { data: vec![0] }
    }

    /// Parse the user strings heap from raw bytes.
    #[must_use]
    pub fn parse(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Get a user string at the given offset.
    pub fn get(&self, offset: u32) -> Result<String> {
        let offset = offset as usize;
        if offset >= self.data.len() {
            return Err(Error::InvalidUserString(offset));
        }

        let mut reader = Reader::new(&self.data[offset..]);
        let blob_len = reader.read_compressed_uint()? as usize;

        if blob_len == 0 {
            return Ok(String::new());
        }

        // The blob length includes a trailing byte indicating if any chars are > 0x7F
        let str_len = blob_len.saturating_sub(1);

        if !str_len.is_multiple_of(2) {
            return Err(Error::InvalidUserString(offset));
        }

        let bytes = reader.read_bytes(str_len)?;

        // Convert UTF-16LE to String
        let utf16: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        String::from_utf16(&utf16).map_err(|_| Error::InvalidUserString(offset))
    }

    /// Add a user string to the heap and return its offset.
    pub fn add(&mut self, s: &str) -> u32 {
        let offset = self.data.len() as u32;

        // Convert to UTF-16LE
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let byte_len = utf16.len() * 2;

        // Calculate if any char has high byte set or is in specific ranges
        let has_special = utf16.iter().any(|&c| {
            c > 0x7F
                || c == 0x01
                || c == 0x02
                || c == 0x03
                || c == 0x04
                || c == 0x05
                || c == 0x06
                || c == 0x07
                || c == 0x08
                || (0x0E..=0x1F).contains(&c)
                || c == 0x27
                || c == 0x2D
        });

        // Blob length = string bytes + 1 (trailing flag byte)
        let blob_len = byte_len + 1;

        // Write compressed length
        let mut writer = Writer::new();
        writer.write_compressed_uint(blob_len as u32);
        self.data.extend_from_slice(writer.as_slice());

        // Write UTF-16LE bytes
        for &c in &utf16 {
            self.data.extend_from_slice(&c.to_le_bytes());
        }

        // Write trailing flag byte
        self.data.push(if has_special { 1 } else { 0 });

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

    /// Write the heap to a writer.
    pub fn write_to(&self, writer: &mut Writer) {
        writer.write_bytes(&self.data);
    }

    /// Write the heap to bytes.
    #[must_use]
    pub fn write(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Iterate over all user strings in the heap with their offsets.
    pub fn iter(&self) -> UserStringsIter<'_> {
        UserStringsIter {
            heap: self,
            offset: 0,
        }
    }
}

impl<'a> IntoIterator for &'a UserStringsHeap {
    type Item = (u32, String);
    type IntoIter = UserStringsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator over user strings in the heap.
///
/// **Note:** This iterator silently stops when it encounters a malformed string
/// (e.g., invalid compressed length or invalid UTF-16 encoding). This is intentional
/// to avoid panicking on corrupt heap data, but callers should be aware that iteration
/// may end early if the heap contains malformed entries.
pub struct UserStringsIter<'a> {
    heap: &'a UserStringsHeap,
    offset: usize,
}

impl Iterator for UserStringsIter<'_> {
    type Item = (u32, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.heap.data.len() {
            return None;
        }

        let start = self.offset;
        let mut reader = Reader::new(&self.heap.data[self.offset..]);
        let blob_len = reader.read_compressed_uint().ok()? as usize;
        let header_size = reader.position();

        if blob_len == 0 {
            self.offset += header_size;
            return Some((start as u32, String::new()));
        }

        // The blob length includes a trailing byte
        let str_len = blob_len.saturating_sub(1);

        if !str_len.is_multiple_of(2) {
            return None;
        }

        let data_end = self.offset + header_size + blob_len;
        if data_end > self.heap.data.len() {
            return None;
        }

        let str_start = self.offset + header_size;
        let bytes = &self.heap.data[str_start..str_start + str_len];

        // Convert UTF-16LE to String
        let utf16: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        let s = String::from_utf16(&utf16).ok()?;

        self.offset = data_end;
        Some((start as u32, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_heap_starts_with_null() {
        let heap = UserStringsHeap::new();
        assert_eq!(heap.size(), 1);
        assert_eq!(heap.data()[0], 0);
    }

    #[test]
    fn test_add_and_get_string() {
        let mut heap = UserStringsHeap::new();
        let offset = heap.add("Hello");
        assert_eq!(heap.get(offset).unwrap(), "Hello");
    }

    #[test]
    fn test_empty_string() {
        let mut heap = UserStringsHeap::new();
        let offset = heap.add("");
        assert_eq!(heap.get(offset).unwrap(), "");
    }

    #[test]
    fn test_unicode_string() {
        let mut heap = UserStringsHeap::new();
        let offset = heap.add("日本語");
        assert_eq!(heap.get(offset).unwrap(), "日本語");
    }

    #[test]
    fn test_parse_heap() {
        // Manually construct: null byte + "Hi" in UTF-16LE with length prefix
        // "Hi" = 0x0048, 0x0069 in UTF-16
        // Length = 4 bytes (2 chars * 2) + 1 (flag) = 5
        let data = [
            0x00, // null byte at start
            0x05, // compressed length = 5
            0x48, 0x00, // 'H'
            0x69, 0x00, // 'i'
            0x00, // flag byte (no special chars)
        ];
        let heap = UserStringsHeap::parse(&data);
        assert_eq!(heap.get(1).unwrap(), "Hi");
    }

    #[test]
    fn test_write_heap() {
        let mut heap = UserStringsHeap::new();
        heap.add("A");
        let data = heap.write();
        // null byte + length (3) + 'A' UTF-16LE (0x41, 0x00) + flag (0)
        assert_eq!(data, vec![0x00, 0x03, 0x41, 0x00, 0x00]);
    }
}
