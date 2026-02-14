//! Coded index types for metadata tables.

use crate::tables::TableId;

/// Kinds of coded indices used in metadata tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        let tag = tables
            .iter()
            .position(|&t| t == self.table)
            .unwrap_or(0) as u32;
        let tag_bits = kind.tag_bits();
        (self.row << tag_bits) | tag
    }
}

