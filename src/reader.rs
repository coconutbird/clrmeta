//! Binary reader for parsing metadata structures.

use crate::error::{Error, Result};

/// A binary reader for parsing little-endian data.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Create a new reader from a byte slice.
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Get the current position.
    #[must_use]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Get the total length of the data.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the reader is at the end.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// Get remaining bytes.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Seek to an absolute position.
    pub fn seek(&mut self, pos: usize) -> Result<()> {
        if pos > self.data.len() {
            return Err(Error::UnexpectedEof {
                offset: pos,
                needed: 0,
            });
        }
        self.pos = pos;
        Ok(())
    }

    /// Read a single byte.
    pub fn read_u8(&mut self) -> Result<u8> {
        if self.pos >= self.data.len() {
            return Err(Error::UnexpectedEof {
                offset: self.pos,
                needed: 1,
            });
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }

    /// Read a little-endian u16.
    pub fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read a little-endian u32.
    pub fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a little-endian u64.
    pub fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read a slice of bytes.
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.pos + len > self.data.len() {
            return Err(Error::UnexpectedEof {
                offset: self.pos,
                needed: len,
            });
        }
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }

    /// Read a null-terminated string.
    pub fn read_null_str(&mut self) -> Result<&'a str> {
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != 0 {
            self.pos += 1;
        }
        if self.pos >= self.data.len() {
            return Err(Error::UnexpectedEof {
                offset: start,
                needed: 1,
            });
        }
        let slice = &self.data[start..self.pos];
        self.pos += 1; // Skip null terminator
        std::str::from_utf8(slice).map_err(|_| Error::InvalidString(start))
    }

    /// Read a 2 or 4 byte index based on size flag.
    pub fn read_index(&mut self, wide: bool) -> Result<u32> {
        if wide {
            self.read_u32()
        } else {
            self.read_u16().map(u32::from)
        }
    }

    /// Read a compressed unsigned integer (ECMA-335 II.23.2).
    pub fn read_compressed_uint(&mut self) -> Result<u32> {
        let start = self.pos;
        let first = self.read_u8()?;

        if first & 0x80 == 0 {
            // 1 byte: 0xxxxxxx
            Ok(u32::from(first))
        } else if first & 0xC0 == 0x80 {
            // 2 bytes: 10xxxxxx xxxxxxxx
            let second = self.read_u8()?;
            Ok(u32::from(first & 0x3F) << 8 | u32::from(second))
        } else if first & 0xE0 == 0xC0 {
            // 4 bytes: 110xxxxx xxxxxxxx xxxxxxxx xxxxxxxx
            let bytes = self.read_bytes(3)?;
            Ok(u32::from(first & 0x1F) << 24
                | u32::from(bytes[0]) << 16
                | u32::from(bytes[1]) << 8
                | u32::from(bytes[2]))
        } else {
            Err(Error::InvalidCompressedInt(start))
        }
    }

    /// Get a sub-reader for a specific range.
    #[must_use]
    pub fn slice(&self, offset: usize, len: usize) -> Option<Reader<'a>> {
        if offset + len <= self.data.len() {
            Some(Reader::new(&self.data[offset..offset + len]))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x42, 0x43, 0x44];
        let mut reader = Reader::new(&data);
        assert_eq!(reader.read_u8().unwrap(), 0x42);
        assert_eq!(reader.read_u8().unwrap(), 0x43);
        assert_eq!(reader.read_u8().unwrap(), 0x44);
        assert!(reader.read_u8().is_err());
    }

    #[test]
    fn test_read_u16() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = Reader::new(&data);
        assert_eq!(reader.read_u16().unwrap(), 0x0201);
        assert_eq!(reader.read_u16().unwrap(), 0x0403);
    }

    #[test]
    fn test_read_u32() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = Reader::new(&data);
        assert_eq!(reader.read_u32().unwrap(), 0x04030201);
    }

    #[test]
    fn test_read_null_str() {
        let data = b"Hello\0World\0";
        let mut reader = Reader::new(data);
        assert_eq!(reader.read_null_str().unwrap(), "Hello");
        assert_eq!(reader.read_null_str().unwrap(), "World");
    }

    #[test]
    fn test_read_compressed_uint_1byte() {
        // Values 0-127 encoded in 1 byte
        let data = [0x00, 0x7F, 0x03];
        let mut reader = Reader::new(&data);
        assert_eq!(reader.read_compressed_uint().unwrap(), 0);
        assert_eq!(reader.read_compressed_uint().unwrap(), 127);
        assert_eq!(reader.read_compressed_uint().unwrap(), 3);
    }

    #[test]
    fn test_read_compressed_uint_2bytes() {
        // Values 128-16383 encoded in 2 bytes: 10xxxxxx xxxxxxxx
        let data = [0x80, 0x80]; // 128
        let mut reader = Reader::new(&data);
        assert_eq!(reader.read_compressed_uint().unwrap(), 128);

        let data2 = [0xBF, 0xFF]; // 16383
        let mut reader2 = Reader::new(&data2);
        assert_eq!(reader2.read_compressed_uint().unwrap(), 16383);
    }

    #[test]
    fn test_read_compressed_uint_4bytes() {
        // Values 16384-536870911 encoded in 4 bytes: 110xxxxx xxxxxxxx xxxxxxxx xxxxxxxx
        let data = [0xC0, 0x00, 0x40, 0x00]; // 16384
        let mut reader = Reader::new(&data);
        assert_eq!(reader.read_compressed_uint().unwrap(), 16384);
    }

    #[test]
    fn test_read_index() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let mut reader = Reader::new(&data);
        // Wide = false: read 2 bytes
        assert_eq!(reader.read_index(false).unwrap(), 0x0201);
        // Wide = true: read 4 bytes
        assert_eq!(reader.read_index(true).unwrap(), 0x06050403);
    }

    #[test]
    fn test_position_and_seek() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = Reader::new(&data);
        assert_eq!(reader.position(), 0);
        reader.read_u16().unwrap();
        assert_eq!(reader.position(), 2);
        reader.seek(0).unwrap();
        assert_eq!(reader.position(), 0);
    }
}
