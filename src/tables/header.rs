//! Tables stream (#~) header parsing and writing.

use crate::error::Result;
use crate::reader::Reader;
use crate::tables::{TableContext, TableId};
use crate::writer::Writer;

/// The tables stream (#~) header.
#[derive(Debug, Clone)]
pub struct TablesHeader {
    /// Reserved (should be 0).
    pub reserved: u32,
    /// Major version (typically 2).
    pub major_version: u8,
    /// Minor version (typically 0).
    pub minor_version: u8,
    /// Heap size flags.
    /// - Bit 0: #Strings uses 4-byte indices
    /// - Bit 1: #GUID uses 4-byte indices
    /// - Bit 2: #Blob uses 4-byte indices
    pub heap_sizes: u8,
    /// Reserved (should be 1).
    pub reserved2: u8,
    /// Bitmask of valid (present) tables.
    pub valid: u64,
    /// Bitmask of sorted tables.
    pub sorted: u64,
    /// Row counts for each valid table.
    pub row_counts: [u32; 64],
}

impl TablesHeader {
    /// Parse the tables header from a reader.
    pub fn parse(reader: &mut Reader<'_>) -> Result<Self> {
        let reserved = reader.read_u32()?;
        let major_version = reader.read_u8()?;
        let minor_version = reader.read_u8()?;
        let heap_sizes = reader.read_u8()?;
        let reserved2 = reader.read_u8()?;
        let valid = reader.read_u64()?;
        let sorted = reader.read_u64()?;

        // Read row counts for each valid table
        let mut row_counts = [0u32; 64];
        for i in 0..64 {
            if valid & (1u64 << i) != 0 {
                row_counts[i] = reader.read_u32()?;
            }
        }

        Ok(Self {
            reserved,
            major_version,
            minor_version,
            heap_sizes,
            reserved2,
            valid,
            sorted,
            row_counts,
        })
    }

    /// Write the tables header to a writer.
    pub fn write_to(&self, writer: &mut Writer) {
        writer.write_u32(self.reserved);
        writer.write_u8(self.major_version);
        writer.write_u8(self.minor_version);
        writer.write_u8(self.heap_sizes);
        writer.write_u8(self.reserved2);
        writer.write_u64(self.valid);
        writer.write_u64(self.sorted);

        // Write row counts for each valid table
        for i in 0..64 {
            if self.valid & (1u64 << i) != 0 {
                writer.write_u32(self.row_counts[i]);
            }
        }
    }

    /// Check if a table is present.
    #[must_use]
    pub fn has_table(&self, table: TableId) -> bool {
        self.valid & (1u64 << (table as u8)) != 0
    }

    /// Get the row count for a table.
    #[must_use]
    pub fn row_count(&self, table: TableId) -> u32 {
        self.row_counts[table as usize]
    }

    /// Set the row count for a table.
    pub fn set_row_count(&mut self, table: TableId, count: u32) {
        let bit = 1u64 << (table as u8);
        if count > 0 {
            self.valid |= bit;
        } else {
            self.valid &= !bit;
        }
        self.row_counts[table as usize] = count;
    }

    /// Create a table context from this header.
    #[must_use]
    pub fn context(&self) -> TableContext {
        TableContext::new(self.heap_sizes, self.row_counts)
    }

    /// Calculate the size of this header in bytes.
    #[must_use]
    pub fn size(&self) -> usize {
        let valid_count = self.valid.count_ones() as usize;
        24 + valid_count * 4 // header(24) + row_counts(4 each)
    }

    /// Iterate over valid tables with their row counts.
    pub fn tables(&self) -> impl Iterator<Item = (TableId, u32)> + '_ {
        (0..64u8).filter_map(move |i| {
            if self.valid & (1u64 << i) != 0 {
                TableId::from_u8(i).ok().map(|t| (t, self.row_counts[i as usize]))
            } else {
                None
            }
        })
    }
}

