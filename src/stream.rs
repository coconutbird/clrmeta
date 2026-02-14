//! Metadata stream header parsing and writing.

use crate::error::Result;
use crate::reader::Reader;
use crate::writer::Writer;

/// A metadata stream header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamHeader {
    /// Offset from the start of the metadata root.
    pub offset: u32,
    /// Size of the stream in bytes.
    pub size: u32,
    /// Stream name (e.g., "#~", "#Strings", "#US", "#GUID", "#Blob").
    pub name: String,
}

impl StreamHeader {
    /// Well-known stream names.
    pub const TABLES: &'static str = "#~";
    pub const TABLES_UNCOMPRESSED: &'static str = "#-";
    pub const STRINGS: &'static str = "#Strings";
    pub const USER_STRINGS: &'static str = "#US";
    pub const GUID: &'static str = "#GUID";
    pub const BLOB: &'static str = "#Blob";

    /// Parse a stream header from the reader.
    pub fn parse(reader: &mut Reader<'_>) -> Result<Self> {
        let offset = reader.read_u32()?;
        let size = reader.read_u32()?;

        // Read null-terminated name
        let name_start = reader.position();
        let name = reader.read_null_str()?.to_string();

        // Stream names are 4-byte aligned (including null terminator)
        let name_len_with_null = reader.position() - name_start;
        let padding = (4 - (name_len_with_null % 4)) % 4;
        if padding > 0 {
            reader.read_bytes(padding)?;
        }

        Ok(Self { offset, size, name })
    }

    /// Write the stream header to a writer.
    pub fn write(&self, writer: &mut Writer) {
        writer.write_u32(self.offset);
        writer.write_u32(self.size);
        writer.write_null_str(&self.name);

        // Align to 4 bytes
        let name_len_with_null = self.name.len() + 1;
        let padding = (4 - (name_len_with_null % 4)) % 4;
        for _ in 0..padding {
            writer.write_u8(0);
        }
    }

    /// Calculate the serialized size of this header.
    #[must_use]
    pub fn serialized_size(&self) -> usize {
        let name_len_with_null = self.name.len() + 1;
        let padding = (4 - (name_len_with_null % 4)) % 4;
        8 + name_len_with_null + padding // offset(4) + size(4) + name + padding
    }

    /// Check if this is the tables stream (#~ or #-).
    #[must_use]
    pub fn is_tables(&self) -> bool {
        self.name == Self::TABLES || self.name == Self::TABLES_UNCOMPRESSED
    }

    /// Check if this is the strings heap.
    #[must_use]
    pub fn is_strings(&self) -> bool {
        self.name == Self::STRINGS
    }

    /// Check if this is the user strings heap.
    #[must_use]
    pub fn is_user_strings(&self) -> bool {
        self.name == Self::USER_STRINGS
    }

    /// Check if this is the GUID heap.
    #[must_use]
    pub fn is_guid(&self) -> bool {
        self.name == Self::GUID
    }

    /// Check if this is the blob heap.
    #[must_use]
    pub fn is_blob(&self) -> bool {
        self.name == Self::BLOB
    }
}

/// Find a stream by name in a list of stream headers.
pub fn find_stream<'a>(streams: &'a [StreamHeader], name: &str) -> Option<&'a StreamHeader> {
    streams.iter().find(|s| s.name == name)
}

/// Find the tables stream (#~ or #-) in a list of stream headers.
pub fn find_tables_stream(streams: &[StreamHeader]) -> Option<&StreamHeader> {
    streams.iter().find(|s| s.is_tables())
}

