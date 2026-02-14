//! Error types for clrmeta.

use thiserror::Error;

/// Result type alias for clrmeta operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during metadata parsing or writing.
#[derive(Debug, Error)]
pub enum Error {
    /// Invalid BSJB signature (expected 0x424A5342).
    #[error("invalid metadata signature: expected 0x424A5342, got 0x{0:08X}")]
    InvalidSignature(u32),

    /// Unexpected end of data while reading.
    #[error("unexpected end of data at offset {offset}, needed {needed} bytes")]
    UnexpectedEof {
        /// Offset where the read was attempted.
        offset: usize,
        /// Number of bytes needed.
        needed: usize,
    },

    /// Invalid or malformed stream name.
    #[error("invalid stream name at offset {0}")]
    InvalidStreamName(usize),

    /// Stream not found by name.
    #[error("stream not found: {0}")]
    StreamNotFound(String),

    /// Invalid UTF-8 string in #Strings heap.
    #[error("invalid UTF-8 string at offset {0}")]
    InvalidString(usize),

    /// Invalid UTF-16 string in #US heap.
    #[error("invalid UTF-16 string at offset {0}")]
    InvalidUserString(usize),

    /// Invalid table ID.
    #[error("invalid table ID: {0}")]
    InvalidTableId(u8),

    /// Invalid coded index.
    #[error("invalid coded index for {kind}: {value}")]
    InvalidCodedIndex {
        /// The kind of coded index.
        kind: &'static str,
        /// The invalid value.
        value: u32,
    },

    /// Invalid compressed integer encoding.
    #[error("invalid compressed integer at offset {0}")]
    InvalidCompressedInt(usize),

    /// Invalid GUID index (out of bounds).
    #[error("invalid GUID index: {0}")]
    InvalidGuidIndex(u32),

    /// Invalid blob data.
    #[error("invalid blob at offset {0}")]
    InvalidBlob(usize),

    /// Table row index out of bounds.
    #[error("table {table} row index {index} out of bounds (max {max})")]
    RowIndexOutOfBounds {
        /// Table name.
        table: &'static str,
        /// Requested index.
        index: u32,
        /// Maximum valid index.
        max: u32,
    },
}

