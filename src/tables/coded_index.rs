//! Coded index types for metadata tables.

use crate::tables::TableId;

/// Kinds of coded indices used in metadata tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CodedIndexKind {
    TypeDefOrRef,
    HasConstant,
    HasCustomAttribute,
    HasFieldMarshal,
    HasDeclSecurity,
    MemberRefParent,
    HasSemantics,
    MethodDefOrRef,
    MemberForwarded,
    Implementation,
    CustomAttributeType,
    ResolutionScope,
    TypeOrMethodDef,
}

impl CodedIndexKind {
    /// Get the number of tag bits for this coded index kind.
    #[must_use]
    pub const fn tag_bits(self) -> u8 {
        match self {
            Self::TypeDefOrRef => 2,
            Self::HasConstant => 2,
            Self::HasCustomAttribute => 5,
            Self::HasFieldMarshal => 1,
            Self::HasDeclSecurity => 2,
            Self::MemberRefParent => 3,
            Self::HasSemantics => 1,
            Self::MethodDefOrRef => 1,
            Self::MemberForwarded => 1,
            Self::Implementation => 2,
            Self::CustomAttributeType => 3,
            Self::ResolutionScope => 2,
            Self::TypeOrMethodDef => 1,
        }
    }

    /// Get the tables that can be referenced by this coded index kind.
    #[must_use]
    pub const fn tables(self) -> &'static [Option<TableId>] {
        match self {
            Self::TypeDefOrRef => &[
                Some(TableId::TypeDef),
                Some(TableId::TypeRef),
                Some(TableId::TypeSpec),
            ],
            Self::HasConstant => &[
                Some(TableId::Field),
                Some(TableId::Param),
                Some(TableId::Property),
            ],
            Self::HasCustomAttribute => &[
                Some(TableId::MethodDef),
                Some(TableId::Field),
                Some(TableId::TypeRef),
                Some(TableId::TypeDef),
                Some(TableId::Param),
                Some(TableId::InterfaceImpl),
                Some(TableId::MemberRef),
                Some(TableId::Module),
                None, // Permission (not used)
                Some(TableId::Property),
                Some(TableId::Event),
                Some(TableId::StandAloneSig),
                Some(TableId::ModuleRef),
                Some(TableId::TypeSpec),
                Some(TableId::Assembly),
                Some(TableId::AssemblyRef),
                Some(TableId::File),
                Some(TableId::ExportedType),
                Some(TableId::ManifestResource),
                Some(TableId::GenericParam),
                Some(TableId::GenericParamConstraint),
                Some(TableId::MethodSpec),
            ],
            Self::HasFieldMarshal => &[Some(TableId::Field), Some(TableId::Param)],
            Self::HasDeclSecurity => &[
                Some(TableId::TypeDef),
                Some(TableId::MethodDef),
                Some(TableId::Assembly),
            ],
            Self::MemberRefParent => &[
                Some(TableId::TypeDef),
                Some(TableId::TypeRef),
                Some(TableId::ModuleRef),
                Some(TableId::MethodDef),
                Some(TableId::TypeSpec),
            ],
            Self::HasSemantics => &[Some(TableId::Event), Some(TableId::Property)],
            Self::MethodDefOrRef => &[Some(TableId::MethodDef), Some(TableId::MemberRef)],
            Self::MemberForwarded => &[Some(TableId::Field), Some(TableId::MethodDef)],
            Self::Implementation => &[
                Some(TableId::File),
                Some(TableId::AssemblyRef),
                Some(TableId::ExportedType),
            ],
            Self::CustomAttributeType => &[
                None, // Not used
                None, // Not used
                Some(TableId::MethodDef),
                Some(TableId::MemberRef),
                None, // Not used
            ],
            Self::ResolutionScope => &[
                Some(TableId::Module),
                Some(TableId::ModuleRef),
                Some(TableId::AssemblyRef),
                Some(TableId::TypeRef),
            ],
            Self::TypeOrMethodDef => &[Some(TableId::TypeDef), Some(TableId::MethodDef)],
        }
    }

    /// Get the maximum number of rows that can use a 2-byte index.
    #[must_use]
    pub const fn max_small_rows(self) -> u32 {
        1u32 << (16 - self.tag_bits())
    }
}

/// A decoded coded index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CodedIndex {
    /// The table this index refers to.
    pub table: Option<TableId>,
    /// The row index (1-based, 0 means null).
    pub row: u32,
}

impl CodedIndex {
    /// Create a null coded index.
    #[must_use]
    pub const fn null() -> Self {
        Self {
            table: None,
            row: 0,
        }
    }

    /// Check if this is a null index.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.row == 0
    }

    /// Decode a coded index value.
    #[must_use]
    pub fn decode(kind: CodedIndexKind, value: u32) -> Self {
        let tag_bits = kind.tag_bits();
        let tag_mask = (1u32 << tag_bits) - 1;
        let tag = (value & tag_mask) as usize;
        let row = value >> tag_bits;

        let tables = kind.tables();
        let table = tables.get(tag).copied().flatten();

        Self { table, row }
    }

    /// Encode this coded index to a value.
    #[must_use]
    pub fn encode(&self, kind: CodedIndexKind) -> u32 {
        let tables = kind.tables();
        let tag = tables.iter().position(|&t| t == self.table).unwrap_or(0) as u32;
        let tag_bits = kind.tag_bits();
        (self.row << tag_bits) | tag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_bits() {
        assert_eq!(CodedIndexKind::TypeDefOrRef.tag_bits(), 2);
        assert_eq!(CodedIndexKind::HasCustomAttribute.tag_bits(), 5);
        assert_eq!(CodedIndexKind::HasFieldMarshal.tag_bits(), 1);
    }

    #[test]
    fn test_null_coded_index() {
        let idx = CodedIndex::null();
        assert!(idx.is_null());
        assert_eq!(idx.row, 0);
        assert_eq!(idx.table, None);
    }

    #[test]
    fn test_decode_type_def_or_ref() {
        // TypeDefOrRef: 2 tag bits
        // Tag 0 = TypeDef, Tag 1 = TypeRef, Tag 2 = TypeSpec

        // Row 5, TypeDef (tag 0): (5 << 2) | 0 = 20
        let idx = CodedIndex::decode(CodedIndexKind::TypeDefOrRef, 20);
        assert_eq!(idx.table, Some(TableId::TypeDef));
        assert_eq!(idx.row, 5);

        // Row 3, TypeRef (tag 1): (3 << 2) | 1 = 13
        let idx = CodedIndex::decode(CodedIndexKind::TypeDefOrRef, 13);
        assert_eq!(idx.table, Some(TableId::TypeRef));
        assert_eq!(idx.row, 3);

        // Row 7, TypeSpec (tag 2): (7 << 2) | 2 = 30
        let idx = CodedIndex::decode(CodedIndexKind::TypeDefOrRef, 30);
        assert_eq!(idx.table, Some(TableId::TypeSpec));
        assert_eq!(idx.row, 7);
    }

    #[test]
    fn test_encode_type_def_or_ref() {
        let idx = CodedIndex {
            table: Some(TableId::TypeDef),
            row: 5,
        };
        assert_eq!(idx.encode(CodedIndexKind::TypeDefOrRef), 20);

        let idx = CodedIndex {
            table: Some(TableId::TypeRef),
            row: 3,
        };
        assert_eq!(idx.encode(CodedIndexKind::TypeDefOrRef), 13);
    }

    #[test]
    fn test_decode_resolution_scope() {
        // ResolutionScope: 2 tag bits
        // Tag 0 = Module, Tag 1 = ModuleRef, Tag 2 = AssemblyRef, Tag 3 = TypeRef

        // Row 1, AssemblyRef (tag 2): (1 << 2) | 2 = 6
        let idx = CodedIndex::decode(CodedIndexKind::ResolutionScope, 6);
        assert_eq!(idx.table, Some(TableId::AssemblyRef));
        assert_eq!(idx.row, 1);
    }

    #[test]
    fn test_roundtrip() {
        let original = CodedIndex {
            table: Some(TableId::MethodDef),
            row: 42,
        };
        let encoded = original.encode(CodedIndexKind::MethodDefOrRef);
        let decoded = CodedIndex::decode(CodedIndexKind::MethodDefOrRef, encoded);
        assert_eq!(decoded.table, original.table);
        assert_eq!(decoded.row, original.row);
    }

    #[test]
    fn test_max_small_rows() {
        // TypeDefOrRef: 2 tag bits -> max 16384 rows for 2-byte index
        assert_eq!(CodedIndexKind::TypeDefOrRef.max_small_rows(), 16384);
        // HasCustomAttribute: 5 tag bits -> max 2048 rows for 2-byte index
        assert_eq!(CodedIndexKind::HasCustomAttribute.max_small_rows(), 2048);
    }
}
