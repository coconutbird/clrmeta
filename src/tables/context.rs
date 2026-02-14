//! Table context for calculating index sizes.

use crate::tables::{CodedIndexKind, TableId};

/// Context for reading/writing metadata tables.
/// Tracks heap sizes and row counts to determine index widths.
#[derive(Debug, Clone)]
pub struct TableContext {
    /// HeapSizes byte from tables header.
    pub heap_sizes: u8,
    /// Row counts for each table (indexed by TableId).
    pub row_counts: [u32; 64],
}

impl TableContext {
    /// Create a new table context.
    #[must_use]
    pub fn new(heap_sizes: u8, row_counts: [u32; 64]) -> Self {
        Self {
            heap_sizes,
            row_counts,
        }
    }

    /// Check if #Strings heap uses 4-byte indices.
    #[must_use]
    pub fn wide_string_indices(&self) -> bool {
        self.heap_sizes & 0x01 != 0
    }

    /// Check if #GUID heap uses 4-byte indices.
    #[must_use]
    pub fn wide_guid_indices(&self) -> bool {
        self.heap_sizes & 0x02 != 0
    }

    /// Check if #Blob heap uses 4-byte indices.
    #[must_use]
    pub fn wide_blob_indices(&self) -> bool {
        self.heap_sizes & 0x04 != 0
    }

    /// Get the size of a string index (2 or 4 bytes).
    #[must_use]
    pub fn string_index_size(&self) -> usize {
        if self.wide_string_indices() { 4 } else { 2 }
    }

    /// Get the size of a GUID index (2 or 4 bytes).
    #[must_use]
    pub fn guid_index_size(&self) -> usize {
        if self.wide_guid_indices() { 4 } else { 2 }
    }

    /// Get the size of a blob index (2 or 4 bytes).
    #[must_use]
    pub fn blob_index_size(&self) -> usize {
        if self.wide_blob_indices() { 4 } else { 2 }
    }

    /// Get the row count for a table.
    #[must_use]
    pub fn row_count(&self, table: TableId) -> u32 {
        self.row_counts[table as usize]
    }

    /// Check if a table index uses 4 bytes (row count > 65535).
    #[must_use]
    pub fn wide_table_index(&self, table: TableId) -> bool {
        self.row_counts[table as usize] > 0xFFFF
    }

    /// Get the size of a table index (2 or 4 bytes).
    #[must_use]
    pub fn table_index_size(&self, table: TableId) -> usize {
        if self.wide_table_index(table) { 4 } else { 2 }
    }

    /// Check if a coded index uses 4 bytes.
    #[must_use]
    pub fn wide_coded_index(&self, kind: CodedIndexKind) -> bool {
        let max_rows = kind.max_small_rows();
        kind.tables()
            .iter()
            .filter_map(|&t| t)
            .any(|t| self.row_counts[t as usize] >= max_rows)
    }

    /// Get the size of a coded index (2 or 4 bytes).
    #[must_use]
    pub fn coded_index_size(&self, kind: CodedIndexKind) -> usize {
        if self.wide_coded_index(kind) { 4 } else { 2 }
    }

    /// Calculate the row size for a given table.
    #[must_use]
    pub fn row_size(&self, table: TableId) -> usize {
        match table {
            TableId::Module => 2 + self.string_index_size() * 2 + self.guid_index_size() * 3,
            TableId::TypeRef => {
                self.coded_index_size(CodedIndexKind::ResolutionScope)
                    + self.string_index_size() * 2
            }
            TableId::TypeDef => {
                4 + self.string_index_size() * 2
                    + self.coded_index_size(CodedIndexKind::TypeDefOrRef)
                    + self.table_index_size(TableId::Field)
                    + self.table_index_size(TableId::MethodDef)
            }
            TableId::Field => 2 + self.string_index_size() + self.blob_index_size(),
            TableId::MethodDef => {
                4 + 2
                    + 2
                    + self.string_index_size()
                    + self.blob_index_size()
                    + self.table_index_size(TableId::Param)
            }
            TableId::Param => 2 + 2 + self.string_index_size(),
            TableId::InterfaceImpl => {
                self.table_index_size(TableId::TypeDef)
                    + self.coded_index_size(CodedIndexKind::TypeDefOrRef)
            }
            TableId::MemberRef => {
                self.coded_index_size(CodedIndexKind::MemberRefParent)
                    + self.string_index_size()
                    + self.blob_index_size()
            }
            TableId::Constant => {
                2 + self.coded_index_size(CodedIndexKind::HasConstant) + self.blob_index_size()
            }
            TableId::CustomAttribute => {
                self.coded_index_size(CodedIndexKind::HasCustomAttribute)
                    + self.coded_index_size(CodedIndexKind::CustomAttributeType)
                    + self.blob_index_size()
            }
            TableId::Assembly => {
                4 + 2 * 4 + 4 + self.blob_index_size() + self.string_index_size() * 2
            }
            TableId::AssemblyRef => {
                2 * 4 + 4 + self.blob_index_size() * 2 + self.string_index_size() * 2
            }
            TableId::FieldMarshal => {
                self.coded_index_size(CodedIndexKind::HasFieldMarshal) + self.blob_index_size()
            }
            TableId::DeclSecurity => {
                2 + self.coded_index_size(CodedIndexKind::HasDeclSecurity) + self.blob_index_size()
            }
            TableId::ClassLayout => 2 + 4 + self.table_index_size(TableId::TypeDef),
            TableId::FieldLayout => 4 + self.table_index_size(TableId::Field),
            TableId::StandAloneSig => self.blob_index_size(),
            TableId::EventMap => {
                self.table_index_size(TableId::TypeDef) + self.table_index_size(TableId::Event)
            }
            TableId::Event => {
                2 + self.string_index_size() + self.coded_index_size(CodedIndexKind::TypeDefOrRef)
            }
            TableId::PropertyMap => {
                self.table_index_size(TableId::TypeDef) + self.table_index_size(TableId::Property)
            }
            TableId::Property => 2 + self.string_index_size() + self.blob_index_size(),
            TableId::MethodSemantics => {
                2 + self.table_index_size(TableId::MethodDef)
                    + self.coded_index_size(CodedIndexKind::HasSemantics)
            }
            TableId::MethodImpl => {
                self.table_index_size(TableId::TypeDef)
                    + self.coded_index_size(CodedIndexKind::MethodDefOrRef) * 2
            }
            TableId::ModuleRef => self.string_index_size(),
            TableId::TypeSpec => self.blob_index_size(),
            TableId::ImplMap => {
                2 + self.coded_index_size(CodedIndexKind::MemberForwarded)
                    + self.string_index_size()
                    + self.table_index_size(TableId::ModuleRef)
            }
            TableId::FieldRva => 4 + self.table_index_size(TableId::Field),
            TableId::NestedClass => self.table_index_size(TableId::TypeDef) * 2,
            TableId::GenericParam => {
                2 + 2
                    + self.coded_index_size(CodedIndexKind::TypeOrMethodDef)
                    + self.string_index_size()
            }
            TableId::MethodSpec => {
                self.coded_index_size(CodedIndexKind::MethodDefOrRef) + self.blob_index_size()
            }
            TableId::GenericParamConstraint => {
                self.table_index_size(TableId::GenericParam)
                    + self.coded_index_size(CodedIndexKind::TypeDefOrRef)
            }
            // Remaining tables return 0 (not implemented)
            _ => 0,
        }
    }
}
