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
            self.resolution_scope
                .encode(CodedIndexKind::ResolutionScope),
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

/// InterfaceImpl table row (0x09).
#[derive(Debug, Clone, Default)]
pub struct InterfaceImplRow {
    /// TypeDef index of the class implementing the interface.
    pub class: u32,
    /// TypeDefOrRef coded index of the interface.
    pub interface: CodedIndex,
}

impl InterfaceImplRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            class: reader.read_index(ctx.wide_table_index(TableId::TypeDef))?,
            interface: CodedIndex::decode(
                CodedIndexKind::TypeDefOrRef,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::TypeDefOrRef))?,
            ),
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_index(self.class, ctx.wide_table_index(TableId::TypeDef));
        writer.write_index(
            self.interface.encode(CodedIndexKind::TypeDefOrRef),
            ctx.wide_coded_index(CodedIndexKind::TypeDefOrRef),
        );
    }
}

/// Constant table row (0x0B).
#[derive(Debug, Clone, Default)]
pub struct ConstantRow {
    /// Element type (one of ELEMENT_TYPE_*).
    pub constant_type: u8,
    /// Padding byte.
    pub padding: u8,
    /// HasConstant coded index (Field, Param, or Property).
    pub parent: CodedIndex,
    /// Value index into #Blob.
    pub value: u32,
}

impl ConstantRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            constant_type: reader.read_u8()?,
            padding: reader.read_u8()?,
            parent: CodedIndex::decode(
                CodedIndexKind::HasConstant,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::HasConstant))?,
            ),
            value: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u8(self.constant_type);
        writer.write_u8(self.padding);
        writer.write_index(
            self.parent.encode(CodedIndexKind::HasConstant),
            ctx.wide_coded_index(CodedIndexKind::HasConstant),
        );
        writer.write_index(self.value, ctx.wide_blob_indices());
    }
}

/// FieldMarshal table row (0x0D).
#[derive(Debug, Clone, Default)]
pub struct FieldMarshalRow {
    /// HasFieldMarshal coded index.
    pub parent: CodedIndex,
    /// Native type index into #Blob.
    pub native_type: u32,
}

impl FieldMarshalRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            parent: CodedIndex::decode(
                CodedIndexKind::HasFieldMarshal,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::HasFieldMarshal))?,
            ),
            native_type: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_index(
            self.parent.encode(CodedIndexKind::HasFieldMarshal),
            ctx.wide_coded_index(CodedIndexKind::HasFieldMarshal),
        );
        writer.write_index(self.native_type, ctx.wide_blob_indices());
    }
}

/// DeclSecurity table row (0x0E).
#[derive(Debug, Clone, Default)]
pub struct DeclSecurityRow {
    /// Security action.
    pub action: u16,
    /// HasDeclSecurity coded index.
    pub parent: CodedIndex,
    /// Permission set index into #Blob.
    pub permission_set: u32,
}

impl DeclSecurityRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            action: reader.read_u16()?,
            parent: CodedIndex::decode(
                CodedIndexKind::HasDeclSecurity,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::HasDeclSecurity))?,
            ),
            permission_set: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u16(self.action);
        writer.write_index(
            self.parent.encode(CodedIndexKind::HasDeclSecurity),
            ctx.wide_coded_index(CodedIndexKind::HasDeclSecurity),
        );
        writer.write_index(self.permission_set, ctx.wide_blob_indices());
    }
}

/// ClassLayout table row (0x0F).
#[derive(Debug, Clone, Default)]
pub struct ClassLayoutRow {
    /// Packing size.
    pub packing_size: u16,
    /// Class size in bytes.
    pub class_size: u32,
    /// TypeDef index.
    pub parent: u32,
}

impl ClassLayoutRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            packing_size: reader.read_u16()?,
            class_size: reader.read_u32()?,
            parent: reader.read_index(ctx.wide_table_index(TableId::TypeDef))?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_u16(self.packing_size);
        writer.write_u32(self.class_size);
        writer.write_index(self.parent, ctx.wide_table_index(TableId::TypeDef));
    }
}

/// FieldLayout table row (0x10).
#[derive(Debug, Clone, Default)]
pub struct FieldLayoutRow {
    /// Field offset.
    pub offset: u32,
    /// Field index.
    pub field: u32,
}

impl FieldLayoutRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            offset: reader.read_u32()?,
            field: reader.read_index(ctx.wide_table_index(TableId::Field))?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_u32(self.offset);
        writer.write_index(self.field, ctx.wide_table_index(TableId::Field));
    }
}

/// StandAloneSig table row (0x11).
#[derive(Debug, Clone, Default)]
pub struct StandAloneSigRow {
    /// Signature index into #Blob.
    pub signature: u32,
}

impl StandAloneSigRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            signature: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_index(self.signature, ctx.wide_blob_indices());
    }
}

/// EventMap table row (0x12).
#[derive(Debug, Clone, Default)]
pub struct EventMapRow {
    /// TypeDef index.
    pub parent: u32,
    /// Event list start index.
    pub event_list: u32,
}

impl EventMapRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            parent: reader.read_index(ctx.wide_table_index(TableId::TypeDef))?,
            event_list: reader.read_index(ctx.wide_table_index(TableId::Event))?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_index(self.parent, ctx.wide_table_index(TableId::TypeDef));
        writer.write_index(self.event_list, ctx.wide_table_index(TableId::Event));
    }
}

/// Event table row (0x14).
#[derive(Debug, Clone, Default)]
pub struct EventRow {
    /// Event flags.
    pub event_flags: u16,
    /// Event name index into #Strings.
    pub name: u32,
    /// TypeDefOrRef coded index for the event type.
    pub event_type: CodedIndex,
}

impl EventRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            event_flags: reader.read_u16()?,
            name: reader.read_index(ctx.wide_string_indices())?,
            event_type: CodedIndex::decode(
                CodedIndexKind::TypeDefOrRef,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::TypeDefOrRef))?,
            ),
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u16(self.event_flags);
        writer.write_index(self.name, ctx.wide_string_indices());
        writer.write_index(
            self.event_type.encode(CodedIndexKind::TypeDefOrRef),
            ctx.wide_coded_index(CodedIndexKind::TypeDefOrRef),
        );
    }
}

/// PropertyMap table row (0x15).
#[derive(Debug, Clone, Default)]
pub struct PropertyMapRow {
    /// TypeDef index.
    pub parent: u32,
    /// Property list start index.
    pub property_list: u32,
}

impl PropertyMapRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            parent: reader.read_index(ctx.wide_table_index(TableId::TypeDef))?,
            property_list: reader.read_index(ctx.wide_table_index(TableId::Property))?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_index(self.parent, ctx.wide_table_index(TableId::TypeDef));
        writer.write_index(self.property_list, ctx.wide_table_index(TableId::Property));
    }
}

/// Property table row (0x17).
#[derive(Debug, Clone, Default)]
pub struct PropertyRow {
    /// Property flags.
    pub flags: u16,
    /// Property name index into #Strings.
    pub name: u32,
    /// Property signature index into #Blob.
    pub property_type: u32,
}

impl PropertyRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            flags: reader.read_u16()?,
            name: reader.read_index(ctx.wide_string_indices())?,
            property_type: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u16(self.flags);
        writer.write_index(self.name, ctx.wide_string_indices());
        writer.write_index(self.property_type, ctx.wide_blob_indices());
    }
}

/// MethodSemantics table row (0x18).
#[derive(Debug, Clone, Default)]
pub struct MethodSemanticsRow {
    /// Semantics flags (setter, getter, other, addon, removeon, fire).
    pub semantics: u16,
    /// MethodDef index.
    pub method: u32,
    /// HasSemantics coded index (Event or Property).
    pub association: CodedIndex,
}

impl MethodSemanticsRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            semantics: reader.read_u16()?,
            method: reader.read_index(ctx.wide_table_index(TableId::MethodDef))?,
            association: CodedIndex::decode(
                CodedIndexKind::HasSemantics,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::HasSemantics))?,
            ),
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_u16(self.semantics);
        writer.write_index(self.method, ctx.wide_table_index(TableId::MethodDef));
        writer.write_index(
            self.association.encode(CodedIndexKind::HasSemantics),
            ctx.wide_coded_index(CodedIndexKind::HasSemantics),
        );
    }
}

/// MethodImpl table row (0x19).
#[derive(Debug, Clone, Default)]
pub struct MethodImplRow {
    /// TypeDef index of the class.
    pub class: u32,
    /// MethodDefOrRef coded index of the implementation.
    pub method_body: CodedIndex,
    /// MethodDefOrRef coded index of the declaration.
    pub method_declaration: CodedIndex,
}

impl MethodImplRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            class: reader.read_index(ctx.wide_table_index(TableId::TypeDef))?,
            method_body: CodedIndex::decode(
                CodedIndexKind::MethodDefOrRef,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::MethodDefOrRef))?,
            ),
            method_declaration: CodedIndex::decode(
                CodedIndexKind::MethodDefOrRef,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::MethodDefOrRef))?,
            ),
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_index(self.class, ctx.wide_table_index(TableId::TypeDef));
        writer.write_index(
            self.method_body.encode(CodedIndexKind::MethodDefOrRef),
            ctx.wide_coded_index(CodedIndexKind::MethodDefOrRef),
        );
        writer.write_index(
            self.method_declaration
                .encode(CodedIndexKind::MethodDefOrRef),
            ctx.wide_coded_index(CodedIndexKind::MethodDefOrRef),
        );
    }
}

/// ModuleRef table row (0x1A).
#[derive(Debug, Clone, Default)]
pub struct ModuleRefRow {
    /// Module name index into #Strings.
    pub name: u32,
}

impl ModuleRefRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            name: reader.read_index(ctx.wide_string_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_index(self.name, ctx.wide_string_indices());
    }
}

/// TypeSpec table row (0x1B).
#[derive(Debug, Clone, Default)]
pub struct TypeSpecRow {
    /// Signature index into #Blob.
    pub signature: u32,
}

impl TypeSpecRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            signature: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_index(self.signature, ctx.wide_blob_indices());
    }
}

/// ImplMap table row (0x1C).
#[derive(Debug, Clone, Default)]
pub struct ImplMapRow {
    /// Mapping flags.
    pub mapping_flags: u16,
    /// MemberForwarded coded index (Field or MethodDef).
    pub member_forwarded: CodedIndex,
    /// Import name index into #Strings.
    pub import_name: u32,
    /// ModuleRef index.
    pub import_scope: u32,
}

impl ImplMapRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            mapping_flags: reader.read_u16()?,
            member_forwarded: CodedIndex::decode(
                CodedIndexKind::MemberForwarded,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::MemberForwarded))?,
            ),
            import_name: reader.read_index(ctx.wide_string_indices())?,
            import_scope: reader.read_index(ctx.wide_table_index(TableId::ModuleRef))?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_u16(self.mapping_flags);
        writer.write_index(
            self.member_forwarded
                .encode(CodedIndexKind::MemberForwarded),
            ctx.wide_coded_index(CodedIndexKind::MemberForwarded),
        );
        writer.write_index(self.import_name, ctx.wide_string_indices());
        writer.write_index(self.import_scope, ctx.wide_table_index(TableId::ModuleRef));
    }
}

/// FieldRva table row (0x1D).
#[derive(Debug, Clone, Default)]
pub struct FieldRvaRow {
    /// RVA of field data.
    pub rva: u32,
    /// Field index.
    pub field: u32,
}

impl FieldRvaRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            rva: reader.read_u32()?,
            field: reader.read_index(ctx.wide_table_index(TableId::Field))?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_u32(self.rva);
        writer.write_index(self.field, ctx.wide_table_index(TableId::Field));
    }
}

/// NestedClass table row (0x29).
#[derive(Debug, Clone, Default)]
pub struct NestedClassRow {
    /// TypeDef index of the nested class.
    pub nested_class: u32,
    /// TypeDef index of the enclosing class.
    pub enclosing_class: u32,
}

impl NestedClassRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            nested_class: reader.read_index(ctx.wide_table_index(TableId::TypeDef))?,
            enclosing_class: reader.read_index(ctx.wide_table_index(TableId::TypeDef))?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_index(self.nested_class, ctx.wide_table_index(TableId::TypeDef));
        writer.write_index(self.enclosing_class, ctx.wide_table_index(TableId::TypeDef));
    }
}

/// GenericParam table row (0x2A).
#[derive(Debug, Clone, Default)]
pub struct GenericParamRow {
    /// Generic parameter index (0-based within the owner's list).
    pub number: u16,
    /// Generic parameter flags.
    pub flags: u16,
    /// TypeOrMethodDef coded index (owner of this generic param).
    pub owner: CodedIndex,
    /// Parameter name index into #Strings.
    pub name: u32,
}

impl GenericParamRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            number: reader.read_u16()?,
            flags: reader.read_u16()?,
            owner: CodedIndex::decode(
                CodedIndexKind::TypeOrMethodDef,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::TypeOrMethodDef))?,
            ),
            name: reader.read_index(ctx.wide_string_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_u16(self.number);
        writer.write_u16(self.flags);
        writer.write_index(
            self.owner.encode(CodedIndexKind::TypeOrMethodDef),
            ctx.wide_coded_index(CodedIndexKind::TypeOrMethodDef),
        );
        writer.write_index(self.name, ctx.wide_string_indices());
    }
}

/// MethodSpec table row (0x2B).
#[derive(Debug, Clone, Default)]
pub struct MethodSpecRow {
    /// MethodDefOrRef coded index.
    pub method: CodedIndex,
    /// Instantiation signature index into #Blob.
    pub instantiation: u32,
}

impl MethodSpecRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        Ok(Self {
            method: CodedIndex::decode(
                CodedIndexKind::MethodDefOrRef,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::MethodDefOrRef))?,
            ),
            instantiation: reader.read_index(ctx.wide_blob_indices())?,
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        writer.write_index(
            self.method.encode(CodedIndexKind::MethodDefOrRef),
            ctx.wide_coded_index(CodedIndexKind::MethodDefOrRef),
        );
        writer.write_index(self.instantiation, ctx.wide_blob_indices());
    }
}

/// GenericParamConstraint table row (0x2C).
#[derive(Debug, Clone, Default)]
pub struct GenericParamConstraintRow {
    /// GenericParam index.
    pub owner: u32,
    /// TypeDefOrRef coded index (the constraint type).
    pub constraint: CodedIndex,
}

impl GenericParamConstraintRow {
    pub fn parse(reader: &mut Reader<'_>, ctx: &TableContext) -> Result<Self> {
        use crate::tables::TableId;
        Ok(Self {
            owner: reader.read_index(ctx.wide_table_index(TableId::GenericParam))?,
            constraint: CodedIndex::decode(
                CodedIndexKind::TypeDefOrRef,
                reader.read_index(ctx.wide_coded_index(CodedIndexKind::TypeDefOrRef))?,
            ),
        })
    }

    pub fn write(&self, writer: &mut Writer, ctx: &TableContext) {
        use crate::tables::TableId;
        writer.write_index(self.owner, ctx.wide_table_index(TableId::GenericParam));
        writer.write_index(
            self.constraint.encode(CodedIndexKind::TypeDefOrRef),
            ctx.wide_coded_index(CodedIndexKind::TypeDefOrRef),
        );
    }
}
