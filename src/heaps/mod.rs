//! Metadata heaps: #Strings, #US, #GUID, #Blob.

mod blob;
mod guid;
mod strings;
mod us;

pub use blob::BlobHeap;
pub use guid::{format_guid, Guid, GuidHeap};
pub use strings::StringsHeap;
pub use us::UserStringsHeap;

