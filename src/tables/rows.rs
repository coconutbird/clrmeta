//! Table row structures.

use crate::error::Result;
use crate::reader::Reader;
use crate::tables::{CodedIndex, CodedIndexKind, TableContext};
use crate::writer::Writer;

/// Module table row (0x00).
#[derive(Debug, Clone, Default)]
pub struct ModuleRow {
    /// Generation (reserved, should be 0).
    pub generation: u16,
    /// Module name index into #Strings.
    pub name: u32,
    /// Module GUID index into #GUID.
    pub mvid: u32,
    /// EncId GUID index (reserved).
    pub enc_id: u32,
    /// EncBaseId GUID index (reserved).
    pub enc_base_id: u32,
}

impl ModuleRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            generation: reader.read_u16()?,
            name: reader.read_index(ctx.wide_string_indices())?,
            mvid: reader.read_index(ctx.wide_guid_indices())?,
            enc_id: reader.read_index(ctx.wide_guid_indices())?,
            enc_base_id: reader.read_index(ctx.wide_guid_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u16(self.generation);
        writer.write_index(self.name, ctx.wide_string_indices());
        writer.write_index(self.mvid, ctx.wide_guid_indices());
        writer.write_index(self.enc_id, ctx.wide_guid_indices());
        writer.write_index(self.enc_base_id, ctx.wide_guid_indices());
    }
}

/// TypeRef table row (0x01).
#[derive(Debug, Clone, Default)]
pub struct TypeRefRow {
    /// ResolutionScope coded index.
    pub resolution_scope: CodedIndex,
    /// Type name index into #Strings.
    pub type_name: u32,
    /// Type namespace index into #Strings.
    pub type_namespace: u32,
}

impl TypeRefRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        let wide = ctx.wide_coded_index(CodedIndexKind::ResolutionScope);
        Ok(Self {
            resolution_scope: CodedIndex::decode(
                CodedIndexKind::ResolutionScope,
                reader.read_index(wide)?,
            ),
            type_name: reader.read_index(ctx.wide_string_indices())?,
            type_namespace: reader.read_index(ctx.wide_string_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        let wide = ctx.wide_coded_index(CodedIndexKind::ResolutionScope);
        writer.write_index(
            self.resolution_scope.encode(CodedIndexKind::ResolutionScope),
            wide,
        );
        writer.write_index(self.type_name, ctx.wide_string_indices());
        writer.write_index(self.type_namespace, ctx.wide_string_indices());
    }
}

/// TypeDef table row (0x02).
#[derive(Debug, Clone, Default)]
pub struct TypeDefRow {
    /// Type attributes (flags).
    pub flags: u32,
    /// Type name index into #Strings.
    pub type_name: u32,
    /// Type namespace index into #Strings.
    pub type_namespace: u32,
    /// Extends coded index (TypeDefOrRef).
    pub extends: CodedIndex,
    /// First field index into Field table.
    pub field_list: u32,
    /// First method index into MethodDef table.
    pub method_list: u32,
}

impl TypeDefRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            flags: reader.read_u32()?,
            type_name: reader.read_index(ctx.wide_string_indices())?,
            type_namespace: reader.read_index(ctx.wide_string_indices())?,
            extends: CodedIndex::decode(
                CodedIndexKind::TypeDefOrRef,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::TypeDefOrRef))?,
            ),
            field_list: reader.read_index(ctx.wide_table_index(TableId::Field))?,
            method_list: reader.read_index(ctx.wide_table_index(TableId::MethodDef))?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_u32(self.flags);
        writer.write_index(self.type_name, ctx.wide_string_indices());
        writer.write_index(self.type_namespace, ctx.wide_string_indices());
        writer.write_index(
            self.extends.encode(CodedIndexKind::TypeDefOrRef),
            ctx.wide_coded_index(CodedIndexKind::TypeDefOrRef),
        );
        writer.write_index(self.field_list, ctx.wide_table_index(TableId::Field));
        writer.write_index(self.method_list, ctx.wide_table_index(TableId::MethodDef));
    }
}

/// Field table row (0x04).
#[derive(Debug, Clone, Default)]
pub struct FieldRow {
    /// Field attributes (flags).
    pub flags: u16,
    /// Field name index into #Strings.
    pub name: u32,
    /// Signature index into #Blob.
    pub signature: u32,
}

impl FieldRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            flags: reader.read_u16()?,
            name: reader.read_index(ctx.wide_string_indices())?,
            signature: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u16(self.flags);
        writer.write_index(self.name, ctx.wide_string_indices());
        writer.write_index(self.signature, ctx.wide_blob_indices());
    }
}

/// MethodDef table row (0x06).
#[derive(Debug, Clone, Default)]
pub struct MethodDefRow {
    /// RVA of the method body.
    pub rva: u32,
    /// Implementation flags.
    pub impl_flags: u16,
    /// Method flags.
    pub flags: u16,
    /// Method name index into #Strings.
    pub name: u32,
    /// Signature index into #Blob.
    pub signature: u32,
    /// First parameter index into Param table.
    pub param_list: u32,
}

impl MethodDefRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            rva: reader.read_u32()?,
            impl_flags: reader.read_u16()?,
            flags: reader.read_u16()?,
            name: reader.read_index(ctx.wide_string_indices())?,
            signature: reader.read_index(ctx.wide_blob_indices())?,
            param_list: reader.read_index(ctx.wide_table_index(TableId::Param))?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_u32(self.rva);
        writer.write_u16(self.impl_flags);
        writer.write_u16(self.flags);
        writer.write_index(self.name, ctx.wide_string_indices());
        writer.write_index(self.signature, ctx.wide_blob_indices());
        writer.write_index(self.param_list, ctx.wide_table_index(TableId::Param));
    }
}

/// Param table row (0x08).
#[derive(Debug, Clone, Default)]
pub struct ParamRow {
    /// Parameter flags.
    pub flags: u16,
    /// Parameter sequence number.
    pub sequence: u16,
    /// Parameter name index into #Strings.
    pub name: u32,
}

impl ParamRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            flags: reader.read_u16()?,
            sequence: reader.read_u16()?,
            name: reader.read_index(ctx.wide_string_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u16(self.flags);
        writer.write_u16(self.sequence);
        writer.write_index(self.name, ctx.wide_string_indices());
    }
}

/// MemberRef table row (0x0A).
#[derive(Debug, Clone, Default)]
pub struct MemberRefRow {
    /// Class coded index (MemberRefParent).
    pub class: CodedIndex,
    /// Member name index into #Strings.
    pub name: u32,
    /// Signature index into #Blob.
    pub signature: u32,
}

impl MemberRefRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            class: CodedIndex::decode(
                CodedIndexKind::MemberRefParent,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::MemberRefParent))?,
            ),
            name: reader.read_index(ctx.wide_string_indices())?,
            signature: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_index(
            self.class.encode(CodedIndexKind::MemberRefParent),
            ctx.wide_coded_index(CodedIndexKind::MemberRefParent),
        );
        writer.write_index(self.name, ctx.wide_string_indices());
        writer.write_index(self.signature, ctx.wide_blob_indices());
    }
}

/// CustomAttribute table row (0x0C).
#[derive(Debug, Clone, Default)]
pub struct CustomAttributeRow {
    /// Parent coded index (HasCustomAttribute).
    pub parent: CodedIndex,
    /// Type coded index (CustomAttributeType).
    pub attr_type: CodedIndex,
    /// Value index into #Blob.
    pub value: u32,
}

impl CustomAttributeRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            parent: CodedIndex::decode(
                CodedIndexKind::HasCustomAttribute,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::HasCustomAttribute))?,
            ),
            attr_type: CodedIndex::decode(
                CodedIndexKind::CustomAttributeType,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::CustomAttributeType))?,
            ),
            value: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_index(
            self.parent.encode(CodedIndexKind::HasCustomAttribute),
            ctx.wide_coded_index(CodedIndexKind::HasCustomAttribute),
        );
        writer.write_index(
            self.attr_type.encode(CodedIndexKind::CustomAttributeType),
            ctx.wide_coded_index(CodedIndexKind::CustomAttributeType),
        );
        writer.write_index(self.value, ctx.wide_blob_indices());
    }
}

/// Assembly table row (0x20).
#[derive(Debug, Clone, Default)]
pub struct AssemblyRow {
    /// Hash algorithm ID.
    pub hash_alg_id: u32,
    /// Major version.
    pub major_version: u16,
    /// Minor version.
    pub minor_version: u16,
    /// Build number.
    pub build_number: u16,
    /// Revision number.
    pub revision_number: u16,
    /// Assembly flags.
    pub flags: u32,
    /// Public key index into #Blob.
    pub public_key: u32,
    /// Assembly name index into #Strings.
    pub name: u32,
    /// Culture index into #Strings.
    pub culture: u32,
}

impl AssemblyRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            hash_alg_id: reader.read_u32()?,
            major_version: reader.read_u16()?,
            minor_version: reader.read_u16()?,
            build_number: reader.read_u16()?,
            revision_number: reader.read_u16()?,
            flags: reader.read_u32()?,
            public_key: reader.read_index(ctx.wide_blob_indices())?,
            name: reader.read_index(ctx.wide_string_indices())?,
            culture: reader.read_index(ctx.wide_string_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u32(self.hash_alg_id);
        writer.write_u16(self.major_version);
        writer.write_u16(self.minor_version);
        writer.write_u16(self.build_number);
        writer.write_u16(self.revision_number);
        writer.write_u32(self.flags);
        writer.write_index(self.public_key, ctx.wide_blob_indices());
        writer.write_index(self.name, ctx.wide_string_indices());
        writer.write_index(self.culture, ctx.wide_string_indices());
    }
}

/// AssemblyRef table row (0x23).
#[derive(Debug, Clone, Default)]
pub struct AssemblyRefRow {
    /// Major version.
    pub major_version: u16,
    /// Minor version.
    pub minor_version: u16,
    /// Build number.
    pub build_number: u16,
    /// Revision number.
    pub revision_number: u16,
    /// Assembly flags.
    pub flags: u32,
    /// Public key or token index into #Blob.
    pub public_key_or_token: u32,
    /// Assembly name index into #Strings.
    pub name: u32,
    /// Culture index into #Strings.
    pub culture: u32,
    /// Hash value index into #Blob.
    pub hash_value: u32,
}

impl AssemblyRefRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            major_version: reader.read_u16()?,
            minor_version: reader.read_u16()?,
            build_number: reader.read_u16()?,
            revision_number: reader.read_u16()?,
            flags: reader.read_u32()?,
            public_key_or_token: reader.read_index(ctx.wide_blob_indices())?,
            name: reader.read_index(ctx.wide_string_indices())?,
            culture: reader.read_index(ctx.wide_string_indices())?,
            hash_value: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u16(self.major_version);
        writer.write_u16(self.minor_version);
        writer.write_u16(self.build_number);
        writer.write_u16(self.revision_number);
        writer.write_u32(self.flags);
        writer.write_index(self.public_key_or_token, ctx.wide_blob_indices());
        writer.write_index(self.name, ctx.wide_string_indices());
        writer.write_index(self.culture, ctx.wide_string_indices());
        writer.write_index(self.hash_value, ctx.wide_blob_indices());
    }
}

