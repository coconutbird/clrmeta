//! Metadata root (BSJB header) parsing and writing.

use crate::error::{Error, Result};
use crate::reader::Reader;
use crate::stream::StreamHeader;
use crate::writer::Writer;

/// BSJB signature (0x424A5342 = "BSJB" in little-endian).
pub const METADATA_SIGNATURE: u32 = 0x424A5342;

/// The metadata root header (BSJB header).
#[derive(Debug, Clone)]
pub struct MetadataRoot {
    /// Major version (typically 1).
    pub major_version: u16,
    /// Minor version (typically 1).
    pub minor_version: u16,
    /// Reserved (should be 0).
    pub reserved: u32,
    /// Runtime version string (e.g., "v4.0.30319").
    pub version: String,
    /// Flags (reserved, should be 0).
    pub flags: u16,
    /// Stream headers.
    pub streams: Vec<StreamHeader>,
}

impl MetadataRoot {
    /// Parse the metadata root from raw bytes.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(data);
        Self::parse_from_reader(&mut reader)
    }

    /// Parse the metadata root from a reader.
    pub fn parse_from_reader(reader: &mut Reader<'_>) -> Result<Self> {
        // Read and validate signature
        let signature = reader.read_u32()?;
        if signature != METADATA_SIGNATURE {
            return Err(Error::InvalidSignature(signature));
        }

        let major_version = reader.read_u16()?;
        let minor_version = reader.read_u16()?;
        let reserved = reader.read_u32()?;

        // Version string length (includes padding to 4-byte boundary)
        let version_len = reader.read_u32()? as usize;
        let version_bytes = reader.read_bytes(version_len)?;

        // Find the actual string (null-terminated within the padded area)
        let version = version_bytes
            .iter()
            .position(|&b| b == 0)
            .map(|pos| &version_bytes[..pos])
            .unwrap_or(version_bytes);
        let version =
            std::str::from_utf8(version).map_err(|_| Error::InvalidString(reader.position()))?;
        let version = version.to_string();

        let flags = reader.read_u16()?;
        let stream_count = reader.read_u16()? as usize;

        let mut streams = Vec::with_capacity(stream_count);
        for _ in 0..stream_count {
            streams.push(StreamHeader::parse(reader)?);
        }

        Ok(Self {
            major_version,
            minor_version,
            reserved,
            version,
            flags,
            streams,
        })
    }

    /// Write the metadata root to bytes.
    #[must_use]
    pub fn write(&self) -> Vec<u8> {
        let mut writer = Writer::new();
        self.write_to(&mut writer);
        writer.into_inner()
    }

    /// Write the metadata root to a writer.
    pub fn write_to(&self, writer: &mut Writer) {
        writer.write_u32(METADATA_SIGNATURE);
        writer.write_u16(self.major_version);
        writer.write_u16(self.minor_version);
        writer.write_u32(self.reserved);

        // Version string padded to 4-byte boundary
        let version_bytes = self.version.as_bytes();
        let version_len_with_null = version_bytes.len() + 1;
        let padded_len = (version_len_with_null + 3) & !3; // Round up to 4

        writer.write_u32(padded_len as u32);
        writer.write_bytes(version_bytes);
        writer.write_u8(0); // Null terminator
        for _ in version_len_with_null..padded_len {
            writer.write_u8(0);
        }

        writer.write_u16(self.flags);
        writer.write_u16(self.streams.len() as u16);

        for stream in &self.streams {
            stream.write(writer);
        }
    }

    /// Calculate the size of the metadata root header (without stream data).
    #[must_use]
    pub fn header_size(&self) -> usize {
        let version_len_with_null = self.version.len() + 1;
        let padded_version_len = (version_len_with_null + 3) & !3;

        // signature(4) + major(2) + minor(2) + reserved(4) + version_len(4) + version + flags(2) + stream_count(2)
        let base = 4 + 2 + 2 + 4 + 4 + padded_version_len + 2 + 2;

        let streams_size: usize = self.streams.iter().map(|s| s.serialized_size()).sum();

        base + streams_size
    }

    /// Find a stream by name.
    #[must_use]
    pub fn find_stream(&self, name: &str) -> Option<&StreamHeader> {
        self.streams.iter().find(|s| s.name == name)
    }

    /// Find the tables stream (#~ or #-).
    #[must_use]
    pub fn tables_stream(&self) -> Option<&StreamHeader> {
        self.streams.iter().find(|s| s.is_tables())
    }
}
