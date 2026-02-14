//! # clrmeta
//!
//! ECMA-335 CLI/.NET metadata parsing library with read/write support.
//!
//! This crate provides functionality to parse, modify, and write CLR metadata
//! from .NET assemblies. It works with raw metadata bytes, making it PE-agnostic
//! and suitable for use with any PE parser.
//!
//! ## Features
//!
//! - Parse BSJB metadata root and stream headers
//! - Access heaps: #Strings, #US, #GUID, #Blob
//! - Parse metadata tables: Module, TypeDef, TypeRef, MethodDef, Assembly, AssemblyRef, etc.
//! - Modify metadata structures
//! - Write metadata back to bytes
//!
//! ## Example
//!
//! ```ignore
//! use clrmeta::Metadata;
//!
//! // Parse metadata from raw bytes
//! let metadata = Metadata::parse(&metadata_bytes)?;
//!
//! println!("Runtime version: {}", metadata.version());
//!
//! if let Some(assembly) = metadata.assembly() {
//!     println!("Assembly: {} v{}.{}.{}.{}",
//!         assembly.name,
//!         assembly.version.0, assembly.version.1,
//!         assembly.version.2, assembly.version.3);
//! }
//!
//! // Modify and write back
//! let modified_bytes = metadata.write();
//! ```

pub mod error;
pub mod heaps;
pub mod metadata;
pub mod reader;
pub mod root;
pub mod stream;
pub mod tables;
pub mod writer;

// Re-export main types
pub use error::{Error, Result};
pub use metadata::{AssemblyInfo, AssemblyRefInfo, Metadata, MethodInfo, TypeInfo};
pub use root::MetadataRoot;
pub use stream::StreamHeader;

// Re-export heaps
pub use heaps::{BlobHeap, GuidHeap, StringsHeap, UserStringsHeap};

// Re-export tables
pub use tables::{CodedIndex, CodedIndexKind, TableId, TablesHeader};

// Re-export table rows
pub use tables::{
    AssemblyRefRow, AssemblyRow, ClassLayoutRow, ConstantRow, CustomAttributeRow, DeclSecurityRow,
    EventMapRow, EventRow, FieldLayoutRow, FieldMarshalRow, FieldRow, FieldRvaRow,
    GenericParamConstraintRow, GenericParamRow, ImplMapRow, InterfaceImplRow, MemberRefRow,
    MethodDefRow, MethodImplRow, MethodSemanticsRow, MethodSpecRow, ModuleRefRow, ModuleRow,
    NestedClassRow, ParamRow, PropertyMapRow, PropertyRow, StandAloneSigRow, TypeDefRow,
    TypeRefRow, TypeSpecRow,
};
