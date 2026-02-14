//! Metadata heaps: #Strings, #US, #GUID, #Blob.

mod blob;
mod guid;
mod strings;
mod us;

pub use blob::BlobHeap;
pub use guid::{Guid, GuidHeap, format_guid};
pub use strings::StringsHeap;
pub use us::UserStringsHeap;
