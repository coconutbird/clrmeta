//! #Strings heap - null-terminated UTF-8 strings.

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::writer::Writer;

/// The #Strings heap containing null-terminated UTF-8 strings.
#[derive(Debug, Clone, Default)]
pub struct StringsHeap {
    /// Raw heap data.
    data: Vec<u8>,
    /// String to offset mapping for O(1) deduplication during writes.
    index_map: HashMap<String, u32>,
}

impl StringsHeap {
    /// Create a new empty strings heap.
    #[must_use]
    pub fn new() -> Self {
        // Heap always starts with a null byte (empty string at index 0)
        let mut index_map = HashMap::new();
        index_map.insert(String::new(), 0);
        Self {
            data: vec![0],
            index_map,
        }
    }

    /// Parse the strings heap from raw bytes.
    #[must_use]
    pub fn parse(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            index_map: HashMap::new(), // Populated lazily or on demand
        }
    }

    /// Get a string at the given offset.
    pub fn get(&self, offset: u32) -> Result<&str> {
        let offset = offset as usize;
        if offset >= self.data.len() {
            return Err(Error::InvalidString(offset));
        }

        // Find the null terminator
        let end = self.data[offset..]
            .iter()
            .position(|&b| b == 0)
            .ok_or(Error::InvalidString(offset))?;

        std::str::from_utf8(&self.data[offset..offset + end])
            .map_err(|_| Error::InvalidString(offset))
    }

    /// Add a string to the heap and return its offset.
    /// Deduplicates strings that already exist in O(1) time.
    pub fn add(&mut self, s: &str) -> u32 {
        // Check if string already exists (O(1) lookup)
        if let Some(&offset) = self.index_map.get(s) {
            return offset;
        }

        // Add new string
        let offset = self.data.len() as u32;
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(0); // Null terminator
        self.index_map.insert(s.to_string(), offset);
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

    /// Iterate over all strings in the heap with their offsets.
    pub fn iter(&self) -> StringsIter<'_> {
        StringsIter {
            heap: self,
            offset: 0,
        }
    }
}

impl<'a> IntoIterator for &'a StringsHeap {
    type Item = (u32, &'a str);
    type IntoIter = StringsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator over strings in the heap.
pub struct StringsIter<'a> {
    heap: &'a StringsHeap,
    offset: usize,
}

impl<'a> Iterator for StringsIter<'a> {
    type Item = (u32, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.heap.data.len() {
            return None;
        }

        let start = self.offset;
        // Find null terminator
        while self.offset < self.heap.data.len() && self.heap.data[self.offset] != 0 {
            self.offset += 1;
        }

        if self.offset < self.heap.data.len() {
            let s = std::str::from_utf8(&self.heap.data[start..self.offset]).ok()?;
            self.offset += 1; // Skip null
            Some((start as u32, s))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_heap_has_empty_string() {
        let heap = StringsHeap::new();
        assert_eq!(heap.get(0).unwrap(), "");
    }

    #[test]
    fn test_add_and_get_string() {
        let mut heap = StringsHeap::new();
        let offset = heap.add("Hello");
        assert_eq!(heap.get(offset).unwrap(), "Hello");
    }

    #[test]
    fn test_string_deduplication() {
        let mut heap = StringsHeap::new();
        let offset1 = heap.add("Test");
        let offset2 = heap.add("Test");
        assert_eq!(offset1, offset2);
    }

    #[test]
    fn test_parse_heap() {
        let data = b"\0Hello\0World\0";
        let heap = StringsHeap::parse(data);
        assert_eq!(heap.get(0).unwrap(), "");
        assert_eq!(heap.get(1).unwrap(), "Hello");
        assert_eq!(heap.get(7).unwrap(), "World");
    }

    #[test]
    fn test_write_heap() {
        let mut heap = StringsHeap::new();
        heap.add("Test");
        let data = heap.write();
        assert_eq!(data, b"\0Test\0");
    }

    #[test]
    fn test_iter() {
        let data = b"\0Hello\0World\0";
        let heap = StringsHeap::parse(data);
        let strings: Vec<_> = heap.iter().collect();
        assert_eq!(strings, vec![(0, ""), (1, "Hello"), (7, "World")]);
    }
}
