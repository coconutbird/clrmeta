//! Metadata tables parsing and writing.

mod coded_index;
mod context;
mod header;
mod rows;
mod table_id;

pub use coded_index::{CodedIndex, CodedIndexKind};
pub use context::TableContext;
pub use header::TablesHeader;
pub use rows::*;
pub use table_id::TableId;

