//! High-level metadata API.

use crate::error::{Error, Result};
use crate::heaps::{BlobHeap, GuidHeap, StringsHeap, UserStringsHeap};
use crate::reader::Reader;
use crate::root::MetadataRoot;
use crate::stream::StreamHeader;
use crate::tables::{
    AssemblyRefRow, AssemblyRow, ClassLayoutRow, ConstantRow, CustomAttributeRow, DeclSecurityRow,
    EncLogRow, EncMapRow, EventMapRow, EventPtrRow, EventRow, FieldLayoutRow, FieldMarshalRow,
    FieldPtrRow, FieldRow, FieldRvaRow, GenericParamConstraintRow, GenericParamRow, ImplMapRow,
    InterfaceImplRow, MemberRefRow, MethodDefRow, MethodImplRow, MethodPtrRow, MethodSemanticsRow,
    MethodSpecRow, ModuleRefRow, ModuleRow, NestedClassRow, ParamPtrRow, ParamRow, PropertyMapRow,
    PropertyPtrRow, PropertyRow, StandAloneSigRow, TableContext, TableId, TablesHeader, TypeDefRow,
    TypeRefRow, TypeSpecRow,
};
use crate::writer::Writer;

/// Parsed CLR metadata with read/write support.
#[derive(Debug, Clone)]
pub struct Metadata {
    /// The metadata root (BSJB header).
    pub root: MetadataRoot,
    /// The #Strings heap.
    pub strings: StringsHeap,
    /// The #US (user strings) heap.
    pub user_strings: UserStringsHeap,
    /// The #GUID heap.
    pub guids: GuidHeap,
    /// The #Blob heap.
    pub blobs: BlobHeap,
    /// The tables header.
    pub tables_header: TablesHeader,

    // Table rows - all tables in order by TableId
    /// Module table rows (0x00).
    pub modules: Vec<ModuleRow>,
    /// TypeRef table rows (0x01).
    pub type_refs: Vec<TypeRefRow>,
    /// TypeDef table rows (0x02).
    pub type_defs: Vec<TypeDefRow>,
    /// FieldPtr table rows (0x03) - only in uncompressed #- streams.
    pub field_ptrs: Vec<FieldPtrRow>,
    /// Field table rows (0x04).
    pub fields: Vec<FieldRow>,
    /// MethodPtr table rows (0x05) - only in uncompressed #- streams.
    pub method_ptrs: Vec<MethodPtrRow>,
    /// MethodDef table rows (0x06).
    pub method_defs: Vec<MethodDefRow>,
    /// ParamPtr table rows (0x07) - only in uncompressed #- streams.
    pub param_ptrs: Vec<ParamPtrRow>,
    /// Param table rows (0x08).
    pub params: Vec<ParamRow>,
    /// InterfaceImpl table rows (0x09).
    pub interface_impls: Vec<InterfaceImplRow>,
    /// MemberRef table rows (0x0A).
    pub member_refs: Vec<MemberRefRow>,
    /// Constant table rows (0x0B).
    pub constants: Vec<ConstantRow>,
    /// CustomAttribute table rows (0x0C).
    pub custom_attributes: Vec<CustomAttributeRow>,
    /// FieldMarshal table rows (0x0D).
    pub field_marshals: Vec<FieldMarshalRow>,
    /// DeclSecurity table rows (0x0E).
    pub decl_securities: Vec<DeclSecurityRow>,
    /// ClassLayout table rows (0x0F).
    pub class_layouts: Vec<ClassLayoutRow>,
    /// FieldLayout table rows (0x10).
    pub field_layouts: Vec<FieldLayoutRow>,
    /// StandAloneSig table rows (0x11).
    pub stand_alone_sigs: Vec<StandAloneSigRow>,
    /// EventMap table rows (0x12).
    pub event_maps: Vec<EventMapRow>,
    /// EventPtr table rows (0x13) - only in uncompressed #- streams.
    pub event_ptrs: Vec<EventPtrRow>,
    /// Event table rows (0x14).
    pub events: Vec<EventRow>,
    /// PropertyMap table rows (0x15).
    pub property_maps: Vec<PropertyMapRow>,
    /// PropertyPtr table rows (0x16) - only in uncompressed #- streams.
    pub property_ptrs: Vec<PropertyPtrRow>,
    /// Property table rows (0x17).
    pub properties: Vec<PropertyRow>,
    /// MethodSemantics table rows (0x18).
    pub method_semantics: Vec<MethodSemanticsRow>,
    /// MethodImpl table rows (0x19).
    pub method_impls: Vec<MethodImplRow>,
    /// ModuleRef table rows (0x1A).
    pub module_refs: Vec<ModuleRefRow>,
    /// TypeSpec table rows (0x1B).
    pub type_specs: Vec<TypeSpecRow>,
    /// ImplMap table rows (0x1C).
    pub impl_maps: Vec<ImplMapRow>,
    /// FieldRva table rows (0x1D).
    pub field_rvas: Vec<FieldRvaRow>,
    /// EncLog table rows (0x1E) - Edit-and-Continue log.
    pub enc_logs: Vec<EncLogRow>,
    /// EncMap table rows (0x1F) - Edit-and-Continue mapping.
    pub enc_maps: Vec<EncMapRow>,
    /// Assembly table rows (0x20, usually 0 or 1).
    pub assemblies: Vec<AssemblyRow>,
    /// AssemblyRef table rows (0x23).
    pub assembly_refs: Vec<AssemblyRefRow>,
    /// NestedClass table rows (0x29).
    pub nested_classes: Vec<NestedClassRow>,
    /// GenericParam table rows (0x2A).
    pub generic_params: Vec<GenericParamRow>,
    /// MethodSpec table rows (0x2B).
    pub method_specs: Vec<MethodSpecRow>,
    /// GenericParamConstraint table rows (0x2C).
    pub generic_param_constraints: Vec<GenericParamConstraintRow>,
}

impl Metadata {
    /// Parse metadata from raw bytes.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let root = MetadataRoot::parse(data)?;

        // Parse heaps
        let strings = Self::parse_heap(&root, data, StreamHeader::STRINGS, StringsHeap::parse)?;
        let user_strings = Self::parse_heap(
            &root,
            data,
            StreamHeader::USER_STRINGS,
            UserStringsHeap::parse,
        )?;
        let guids = Self::parse_heap(&root, data, StreamHeader::GUID, GuidHeap::parse)?;
        let blobs = Self::parse_heap(&root, data, StreamHeader::BLOB, BlobHeap::parse)?;

        // Parse tables stream (either #~ compressed or #- uncompressed)
        let tables_stream = root
            .tables_stream()
            .ok_or_else(|| Error::StreamNotFound(StreamHeader::TABLES.to_string()))?;
        let uncompressed = tables_stream.name == StreamHeader::TABLES_UNCOMPRESSED;
        let tables_data = &data
            [tables_stream.offset as usize..(tables_stream.offset + tables_stream.size) as usize];
        let mut reader = Reader::new(tables_data);
        let tables_header = TablesHeader::parse(&mut reader, uncompressed)?;
        let ctx = tables_header.context();

        // Parse all tables in order (tables must be read sequentially)
        // 0x00 Module
        let modules = Self::parse_table(&mut reader, &ctx, TableId::Module, ModuleRow::parse)?;
        // 0x01 TypeRef
        let type_refs = Self::parse_table(&mut reader, &ctx, TableId::TypeRef, TypeRefRow::parse)?;
        // 0x02 TypeDef
        let type_defs = Self::parse_table(&mut reader, &ctx, TableId::TypeDef, TypeDefRow::parse)?;
        // 0x03 FieldPtr (only in uncompressed #- streams)
        let field_ptrs =
            Self::parse_table(&mut reader, &ctx, TableId::FieldPtr, FieldPtrRow::parse)?;
        // 0x04 Field
        let fields = Self::parse_table(&mut reader, &ctx, TableId::Field, FieldRow::parse)?;
        // 0x05 MethodPtr (only in uncompressed #- streams)
        let method_ptrs =
            Self::parse_table(&mut reader, &ctx, TableId::MethodPtr, MethodPtrRow::parse)?;
        // 0x06 MethodDef
        let method_defs =
            Self::parse_table(&mut reader, &ctx, TableId::MethodDef, MethodDefRow::parse)?;
        // 0x07 ParamPtr (only in uncompressed #- streams)
        let param_ptrs =
            Self::parse_table(&mut reader, &ctx, TableId::ParamPtr, ParamPtrRow::parse)?;
        // 0x08 Param
        let params = Self::parse_table(&mut reader, &ctx, TableId::Param, ParamRow::parse)?;
        // 0x09 InterfaceImpl
        let interface_impls = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::InterfaceImpl,
            InterfaceImplRow::parse,
        )?;
        // 0x0A MemberRef
        let member_refs =
            Self::parse_table(&mut reader, &ctx, TableId::MemberRef, MemberRefRow::parse)?;
        // 0x0B Constant
        let constants =
            Self::parse_table(&mut reader, &ctx, TableId::Constant, ConstantRow::parse)?;
        // 0x0C CustomAttribute
        let custom_attributes = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::CustomAttribute,
            CustomAttributeRow::parse,
        )?;
        // 0x0D FieldMarshal
        let field_marshals = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::FieldMarshal,
            FieldMarshalRow::parse,
        )?;
        // 0x0E DeclSecurity
        let decl_securities = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::DeclSecurity,
            DeclSecurityRow::parse,
        )?;
        // 0x0F ClassLayout
        let class_layouts = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::ClassLayout,
            ClassLayoutRow::parse,
        )?;
        // 0x10 FieldLayout
        let field_layouts = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::FieldLayout,
            FieldLayoutRow::parse,
        )?;
        // 0x11 StandAloneSig
        let stand_alone_sigs = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::StandAloneSig,
            StandAloneSigRow::parse,
        )?;
        // 0x12 EventMap
        let event_maps =
            Self::parse_table(&mut reader, &ctx, TableId::EventMap, EventMapRow::parse)?;
        // 0x13 EventPtr (only in uncompressed #- streams)
        let event_ptrs =
            Self::parse_table(&mut reader, &ctx, TableId::EventPtr, EventPtrRow::parse)?;
        // 0x14 Event
        let events = Self::parse_table(&mut reader, &ctx, TableId::Event, EventRow::parse)?;
        // 0x15 PropertyMap
        let property_maps = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::PropertyMap,
            PropertyMapRow::parse,
        )?;
        // 0x16 PropertyPtr (only in uncompressed #- streams)
        let property_ptrs =
            Self::parse_table(&mut reader, &ctx, TableId::PropertyPtr, PropertyPtrRow::parse)?;
        // 0x17 Property
        let properties =
            Self::parse_table(&mut reader, &ctx, TableId::Property, PropertyRow::parse)?;
        // 0x18 MethodSemantics
        let method_semantics = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::MethodSemantics,
            MethodSemanticsRow::parse,
        )?;
        // 0x19 MethodImpl
        let method_impls =
            Self::parse_table(&mut reader, &ctx, TableId::MethodImpl, MethodImplRow::parse)?;
        // 0x1A ModuleRef
        let module_refs =
            Self::parse_table(&mut reader, &ctx, TableId::ModuleRef, ModuleRefRow::parse)?;
        // 0x1B TypeSpec
        let type_specs =
            Self::parse_table(&mut reader, &ctx, TableId::TypeSpec, TypeSpecRow::parse)?;
        // 0x1C ImplMap
        let impl_maps = Self::parse_table(&mut reader, &ctx, TableId::ImplMap, ImplMapRow::parse)?;
        // 0x1D FieldRva
        let field_rvas =
            Self::parse_table(&mut reader, &ctx, TableId::FieldRva, FieldRvaRow::parse)?;
        // 0x1E EncLog
        let enc_logs = Self::parse_table(&mut reader, &ctx, TableId::EncLog, EncLogRow::parse)?;
        // 0x1F EncMap
        let enc_maps = Self::parse_table(&mut reader, &ctx, TableId::EncMap, EncMapRow::parse)?;
        // 0x20 Assembly
        let assemblies =
            Self::parse_table(&mut reader, &ctx, TableId::Assembly, AssemblyRow::parse)?;
        // 0x21 AssemblyProcessor (skip)
        Self::skip_table(&mut reader, &ctx, TableId::AssemblyProcessor)?;
        // 0x22 AssemblyOs (skip)
        Self::skip_table(&mut reader, &ctx, TableId::AssemblyOs)?;
        // 0x23 AssemblyRef
        let assembly_refs = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::AssemblyRef,
            AssemblyRefRow::parse,
        )?;
        // 0x24 AssemblyRefProcessor (skip)
        Self::skip_table(&mut reader, &ctx, TableId::AssemblyRefProcessor)?;
        // 0x25 AssemblyRefOs (skip)
        Self::skip_table(&mut reader, &ctx, TableId::AssemblyRefOs)?;
        // 0x26 File (skip)
        Self::skip_table(&mut reader, &ctx, TableId::File)?;
        // 0x27 ExportedType (skip)
        Self::skip_table(&mut reader, &ctx, TableId::ExportedType)?;
        // 0x28 ManifestResource (skip)
        Self::skip_table(&mut reader, &ctx, TableId::ManifestResource)?;
        // 0x29 NestedClass
        let nested_classes = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::NestedClass,
            NestedClassRow::parse,
        )?;
        // 0x2A GenericParam
        let generic_params = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::GenericParam,
            GenericParamRow::parse,
        )?;
        // 0x2B MethodSpec
        let method_specs =
            Self::parse_table(&mut reader, &ctx, TableId::MethodSpec, MethodSpecRow::parse)?;
        // 0x2C GenericParamConstraint
        let generic_param_constraints = Self::parse_table(
            &mut reader,
            &ctx,
            TableId::GenericParamConstraint,
            GenericParamConstraintRow::parse,
        )?;

        Ok(Self {
            root,
            strings,
            user_strings,
            guids,
            blobs,
            tables_header,
            modules,
            type_refs,
            type_defs,
            field_ptrs,
            fields,
            method_ptrs,
            method_defs,
            param_ptrs,
            params,
            interface_impls,
            member_refs,
            constants,
            custom_attributes,
            field_marshals,
            decl_securities,
            class_layouts,
            field_layouts,
            stand_alone_sigs,
            event_maps,
            event_ptrs,
            events,
            property_maps,
            property_ptrs,
            properties,
            method_semantics,
            method_impls,
            module_refs,
            type_specs,
            impl_maps,
            field_rvas,
            enc_logs,
            enc_maps,
            assemblies,
            assembly_refs,
            nested_classes,
            generic_params,
            method_specs,
            generic_param_constraints,
        })
    }

    fn parse_heap<T, F>(root: &MetadataRoot, data: &[u8], name: &str, parser: F) -> Result<T>
    where
        F: FnOnce(&[u8]) -> T,
        T: Default,
    {
        if let Some(stream) = root.find_stream(name) {
            let start = stream.offset as usize;
            let end = start + stream.size as usize;
            if end <= data.len() {
                return Ok(parser(&data[start..end]));
            }
        }
        Ok(T::default())
    }

    fn parse_table<T, F>(
        reader: &mut Reader<'_>,
        ctx: &TableContext,
        table: TableId,
        parser: F,
    ) -> Result<Vec<T>>
    where
        F: Fn(&mut Reader<'_>, &TableContext) -> Result<T>,
    {
        let count = ctx.row_count(table) as usize;
        let mut rows = Vec::with_capacity(count);
        for _ in 0..count {
            rows.push(parser(reader, ctx)?);
        }
        Ok(rows)
    }

    fn skip_table(reader: &mut Reader<'_>, ctx: &TableContext, table: TableId) -> Result<()> {
        let count = ctx.row_count(table) as usize;
        let row_size = ctx.row_size(table);
        reader.read_bytes(count * row_size)?;
        Ok(())
    }

    /// Get the runtime version string.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.root.version
    }

    /// Get assembly information if this is an assembly (not a netmodule).
    #[must_use]
    pub fn assembly(&self) -> Option<AssemblyInfo> {
        self.assemblies.first().map(|row| {
            let name = self.strings.get(row.name).unwrap_or("").to_string();
            let culture = if row.culture != 0 {
                self.strings.get(row.culture).ok().map(|s| s.to_string())
            } else {
                None
            };
            let public_key = if row.public_key != 0 {
                self.blobs.get(row.public_key).ok().map(|b| b.to_vec())
            } else {
                None
            };

            AssemblyInfo {
                name,
                version: (
                    row.major_version,
                    row.minor_version,
                    row.build_number,
                    row.revision_number,
                ),
                culture,
                public_key,
                flags: row.flags,
                hash_alg_id: row.hash_alg_id,
            }
        })
    }

    /// Get all type definitions.
    pub fn types(&self) -> Vec<TypeInfo> {
        self.type_defs
            .iter()
            .map(|row| {
                let name = self.strings.get(row.type_name).unwrap_or("").to_string();
                let namespace = if row.type_namespace != 0 {
                    self.strings
                        .get(row.type_namespace)
                        .ok()
                        .map(|s| s.to_string())
                } else {
                    None
                };
                TypeInfo {
                    name,
                    namespace,
                    flags: row.flags,
                }
            })
            .collect()
    }

    /// Get all method definitions.
    pub fn methods(&self) -> Vec<MethodInfo> {
        self.method_defs
            .iter()
            .map(|row| {
                let name = self.strings.get(row.name).unwrap_or("").to_string();
                MethodInfo {
                    name,
                    rva: row.rva,
                    flags: row.flags,
                    impl_flags: row.impl_flags,
                }
            })
            .collect()
    }

    /// Get all assembly references.
    pub fn assembly_refs(&self) -> Vec<AssemblyRefInfo> {
        self.assembly_refs
            .iter()
            .map(|row| {
                let name = self.strings.get(row.name).unwrap_or("").to_string();
                let culture = if row.culture != 0 {
                    self.strings.get(row.culture).ok().map(|s| s.to_string())
                } else {
                    None
                };
                let public_key_token = if row.public_key_or_token != 0 {
                    self.blobs
                        .get(row.public_key_or_token)
                        .ok()
                        .map(|b| b.to_vec())
                } else {
                    None
                };

                AssemblyRefInfo {
                    name,
                    version: (
                        row.major_version,
                        row.minor_version,
                        row.build_number,
                        row.revision_number,
                    ),
                    culture,
                    public_key_token,
                    flags: row.flags,
                }
            })
            .collect()
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate the metadata structure.
    ///
    /// Performs structural integrity checks including:
    /// - Required tables are present (Module must have at least 1 row)
    /// - String indices are within bounds
    /// - GUID indices are within bounds
    /// - Blob indices are within bounds
    /// - Table index references are valid
    ///
    /// Returns a list of validation errors. An empty list means the metadata is valid.
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check required tables
        if self.modules.is_empty() {
            errors.push("Module table must have at least 1 row".to_string());
        }

        // Validate Module table
        for (i, row) in self.modules.iter().enumerate() {
            self.validate_string_index(&mut errors, "Module", i, "name", row.name);
            self.validate_guid_index(&mut errors, "Module", i, "mvid", row.mvid);
        }

        // Validate TypeRef table
        for (i, row) in self.type_refs.iter().enumerate() {
            self.validate_string_index(&mut errors, "TypeRef", i, "type_name", row.type_name);
            self.validate_string_index(
                &mut errors,
                "TypeRef",
                i,
                "type_namespace",
                row.type_namespace,
            );
        }

        // Validate TypeDef table
        for (i, row) in self.type_defs.iter().enumerate() {
            self.validate_string_index(&mut errors, "TypeDef", i, "type_name", row.type_name);
            self.validate_string_index(
                &mut errors,
                "TypeDef",
                i,
                "type_namespace",
                row.type_namespace,
            );
            self.validate_table_index(
                &mut errors,
                "TypeDef",
                i,
                "field_list",
                row.field_list,
                self.fields.len(),
            );
            self.validate_table_index(
                &mut errors,
                "TypeDef",
                i,
                "method_list",
                row.method_list,
                self.method_defs.len(),
            );
        }

        // Validate Field table
        for (i, row) in self.fields.iter().enumerate() {
            self.validate_string_index(&mut errors, "Field", i, "name", row.name);
            self.validate_blob_index(&mut errors, "Field", i, "signature", row.signature);
        }

        // Validate MethodDef table
        for (i, row) in self.method_defs.iter().enumerate() {
            self.validate_string_index(&mut errors, "MethodDef", i, "name", row.name);
            self.validate_blob_index(&mut errors, "MethodDef", i, "signature", row.signature);
            self.validate_table_index(
                &mut errors,
                "MethodDef",
                i,
                "param_list",
                row.param_list,
                self.params.len(),
            );
        }

        // Validate Param table
        for (i, row) in self.params.iter().enumerate() {
            self.validate_string_index(&mut errors, "Param", i, "name", row.name);
        }

        // Validate MemberRef table
        for (i, row) in self.member_refs.iter().enumerate() {
            self.validate_string_index(&mut errors, "MemberRef", i, "name", row.name);
            self.validate_blob_index(&mut errors, "MemberRef", i, "signature", row.signature);
        }

        // Validate Constant table
        for (i, row) in self.constants.iter().enumerate() {
            self.validate_blob_index(&mut errors, "Constant", i, "value", row.value);
        }

        // Validate CustomAttribute table
        for (i, row) in self.custom_attributes.iter().enumerate() {
            self.validate_blob_index(&mut errors, "CustomAttribute", i, "value", row.value);
        }

        // Validate Assembly table
        for (i, row) in self.assemblies.iter().enumerate() {
            self.validate_string_index(&mut errors, "Assembly", i, "name", row.name);
            self.validate_string_index(&mut errors, "Assembly", i, "culture", row.culture);
            self.validate_blob_index(&mut errors, "Assembly", i, "public_key", row.public_key);
        }

        // Validate AssemblyRef table
        for (i, row) in self.assembly_refs.iter().enumerate() {
            self.validate_string_index(&mut errors, "AssemblyRef", i, "name", row.name);
            self.validate_string_index(&mut errors, "AssemblyRef", i, "culture", row.culture);
            self.validate_blob_index(
                &mut errors,
                "AssemblyRef",
                i,
                "public_key_or_token",
                row.public_key_or_token,
            );
            self.validate_blob_index(&mut errors, "AssemblyRef", i, "hash_value", row.hash_value);
        }

        errors
    }

    /// Validate that the metadata is structurally correct.
    ///
    /// Returns `Ok(())` if valid, or `Err` with the first validation error.
    pub fn validate_strict(&self) -> Result<()> {
        let errors = self.validate();
        if let Some(first_error) = errors.into_iter().next() {
            Err(Error::ValidationError(first_error))
        } else {
            Ok(())
        }
    }

    fn validate_string_index(
        &self,
        errors: &mut Vec<String>,
        table: &str,
        row: usize,
        field: &str,
        index: u32,
    ) {
        if index != 0 && self.strings.get(index).is_err() {
            errors.push(format!(
                "{table}[{row}].{field}: invalid string index {index}"
            ));
        }
    }

    fn validate_guid_index(
        &self,
        errors: &mut Vec<String>,
        table: &str,
        row: usize,
        field: &str,
        index: u32,
    ) {
        if index != 0 && self.guids.get(index).is_err() {
            errors.push(format!(
                "{table}[{row}].{field}: invalid GUID index {index}"
            ));
        }
    }

    fn validate_blob_index(
        &self,
        errors: &mut Vec<String>,
        table: &str,
        row: usize,
        field: &str,
        index: u32,
    ) {
        if index != 0 && self.blobs.get(index).is_err() {
            errors.push(format!(
                "{table}[{row}].{field}: invalid blob index {index}"
            ));
        }
    }

    fn validate_table_index(
        &self,
        errors: &mut Vec<String>,
        table: &str,
        row: usize,
        field: &str,
        index: u32,
        max_rows: usize,
    ) {
        // Table indices are 1-based, 0 means null
        // A "list" index can be max_rows + 1 (meaning empty list at end)
        if index > (max_rows as u32) + 1 {
            errors.push(format!(
                "{table}[{row}].{field}: invalid table index {index} (max {max_rows})"
            ));
        }
    }
}

/// High-level assembly information.
#[derive(Debug, Clone)]
pub struct AssemblyInfo {
    /// Assembly name.
    pub name: String,
    /// Version (major, minor, build, revision).
    pub version: (u16, u16, u16, u16),
    /// Culture (e.g., "en-US"), or None for neutral.
    pub culture: Option<String>,
    /// Public key blob.
    pub public_key: Option<Vec<u8>>,
    /// Assembly flags.
    pub flags: u32,
    /// Hash algorithm ID.
    pub hash_alg_id: u32,
}

impl AssemblyInfo {
    /// Get a formatted version string (e.g., "1.2.3.4").
    #[must_use]
    pub fn version_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.version.0, self.version.1, self.version.2, self.version.3
        )
    }

    /// Compute the public key token (last 8 bytes of SHA-1 hash, reversed).
    #[must_use]
    pub fn public_key_token(&self) -> Option<[u8; 8]> {
        // Note: Requires SHA-1 hashing which we don't implement here
        // Return None for now - users can compute this externally
        None
    }
}

/// High-level type information.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// Type name.
    pub name: String,
    /// Namespace (None if empty).
    pub namespace: Option<String>,
    /// Type attributes/flags.
    pub flags: u32,
}

impl TypeInfo {
    /// Get the full name (namespace.name or just name).
    #[must_use]
    pub fn full_name(&self) -> String {
        if let Some(ns) = &self.namespace {
            if !ns.is_empty() {
                return format!("{}.{}", ns, self.name);
            }
        }
        self.name.clone()
    }
}

/// High-level method information.
#[derive(Debug, Clone)]
pub struct MethodInfo {
    /// Method name.
    pub name: String,
    /// RVA of the method body (0 for abstract/runtime methods).
    pub rva: u32,
    /// Method flags.
    pub flags: u16,
    /// Implementation flags.
    pub impl_flags: u16,
}

/// High-level assembly reference information.
#[derive(Debug, Clone)]
pub struct AssemblyRefInfo {
    /// Assembly name.
    pub name: String,
    /// Version (major, minor, build, revision).
    pub version: (u16, u16, u16, u16),
    /// Culture (e.g., "en-US"), or None for neutral.
    pub culture: Option<String>,
    /// Public key token.
    pub public_key_token: Option<Vec<u8>>,
    /// Assembly flags.
    pub flags: u32,
}

impl AssemblyRefInfo {
    /// Get a formatted version string (e.g., "1.2.3.4").
    #[must_use]
    pub fn version_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.version.0, self.version.1, self.version.2, self.version.3
        )
    }
}

impl Metadata {
    /// Write the metadata to bytes.
    ///
    /// Note: This is a simplified write that may not produce byte-identical output
    /// for complex metadata. It's suitable for modified metadata that will be
    /// re-embedded into a PE file.
    #[must_use]
    pub fn write(&self) -> Vec<u8> {
        let mut writer = Writer::new();
        self.write_to(&mut writer);
        writer.into_inner()
    }

    /// Write the metadata to a writer.
    pub fn write_to(&self, writer: &mut Writer) {
        // For now, we'll write the original structure back
        // A full implementation would rebuild all streams and tables

        // Calculate heap sizes
        let heap_sizes = self.calculate_heap_sizes();

        // Build a modified root with correct offsets
        let mut root = self.root.clone();

        // Calculate stream offsets
        let header_size = root.header_size();
        let mut current_offset = header_size;

        // Update stream headers with new offsets
        for stream in &mut root.streams {
            stream.offset = current_offset as u32;
            match stream.name.as_str() {
                StreamHeader::TABLES | StreamHeader::TABLES_UNCOMPRESSED => {
                    // Tables stream size will be calculated
                    stream.size = self.calculate_tables_size() as u32;
                }
                StreamHeader::STRINGS => {
                    stream.size = self.strings.size() as u32;
                }
                StreamHeader::USER_STRINGS => {
                    stream.size = self.user_strings.size() as u32;
                }
                StreamHeader::GUID => {
                    stream.size = self.guids.size() as u32;
                }
                StreamHeader::BLOB => {
                    stream.size = self.blobs.size() as u32;
                }
                _ => {}
            }
            current_offset += stream.size as usize;
            // Align to 4 bytes
            current_offset = (current_offset + 3) & !3;
        }

        // Write root header
        root.write_to(writer);

        // Write streams in order
        for stream in &root.streams {
            match stream.name.as_str() {
                StreamHeader::TABLES | StreamHeader::TABLES_UNCOMPRESSED => {
                    self.write_tables(writer, heap_sizes);
                }
                StreamHeader::STRINGS => {
                    self.strings.write_to(writer);
                }
                StreamHeader::USER_STRINGS => {
                    self.user_strings.write_to(writer);
                }
                StreamHeader::GUID => {
                    self.guids.write_to(writer);
                }
                StreamHeader::BLOB => {
                    self.blobs.write_to(writer);
                }
                _ => {
                    // Unknown stream - skip
                }
            }
            // Align to 4 bytes
            writer.align(4);
        }
    }

    fn calculate_heap_sizes(&self) -> u8 {
        let mut heap_sizes = 0u8;
        if self.strings.uses_wide_indices() {
            heap_sizes |= 0x01;
        }
        if self.guids.uses_wide_indices() {
            heap_sizes |= 0x02;
        }
        if self.blobs.uses_wide_indices() {
            heap_sizes |= 0x04;
        }
        heap_sizes
    }

    fn calculate_tables_size(&self) -> usize {
        let ctx = self.tables_header.context();

        // Header size
        let mut size = self.tables_header.size();

        // Add size of each table
        for (table, count) in self.tables_header.tables() {
            size += count as usize * ctx.row_size(table);
        }

        size
    }

    fn write_tables(&self, writer: &mut Writer, heap_sizes: u8) {
        // Write tables header
        let mut header = self.tables_header.clone();
        header.heap_sizes = heap_sizes;

        // Update row counts for all tables
        header.set_row_count(TableId::Module, self.modules.len() as u32);
        header.set_row_count(TableId::TypeRef, self.type_refs.len() as u32);
        header.set_row_count(TableId::TypeDef, self.type_defs.len() as u32);
        header.set_row_count(TableId::FieldPtr, self.field_ptrs.len() as u32);
        header.set_row_count(TableId::Field, self.fields.len() as u32);
        header.set_row_count(TableId::MethodPtr, self.method_ptrs.len() as u32);
        header.set_row_count(TableId::MethodDef, self.method_defs.len() as u32);
        header.set_row_count(TableId::ParamPtr, self.param_ptrs.len() as u32);
        header.set_row_count(TableId::Param, self.params.len() as u32);
        header.set_row_count(TableId::InterfaceImpl, self.interface_impls.len() as u32);
        header.set_row_count(TableId::MemberRef, self.member_refs.len() as u32);
        header.set_row_count(TableId::Constant, self.constants.len() as u32);
        header.set_row_count(
            TableId::CustomAttribute,
            self.custom_attributes.len() as u32,
        );
        header.set_row_count(TableId::FieldMarshal, self.field_marshals.len() as u32);
        header.set_row_count(TableId::DeclSecurity, self.decl_securities.len() as u32);
        header.set_row_count(TableId::ClassLayout, self.class_layouts.len() as u32);
        header.set_row_count(TableId::FieldLayout, self.field_layouts.len() as u32);
        header.set_row_count(TableId::StandAloneSig, self.stand_alone_sigs.len() as u32);
        header.set_row_count(TableId::EventMap, self.event_maps.len() as u32);
        header.set_row_count(TableId::EventPtr, self.event_ptrs.len() as u32);
        header.set_row_count(TableId::Event, self.events.len() as u32);
        header.set_row_count(TableId::PropertyMap, self.property_maps.len() as u32);
        header.set_row_count(TableId::PropertyPtr, self.property_ptrs.len() as u32);
        header.set_row_count(TableId::Property, self.properties.len() as u32);
        header.set_row_count(TableId::MethodSemantics, self.method_semantics.len() as u32);
        header.set_row_count(TableId::MethodImpl, self.method_impls.len() as u32);
        header.set_row_count(TableId::ModuleRef, self.module_refs.len() as u32);
        header.set_row_count(TableId::TypeSpec, self.type_specs.len() as u32);
        header.set_row_count(TableId::ImplMap, self.impl_maps.len() as u32);
        header.set_row_count(TableId::FieldRva, self.field_rvas.len() as u32);
        header.set_row_count(TableId::EncLog, self.enc_logs.len() as u32);
        header.set_row_count(TableId::EncMap, self.enc_maps.len() as u32);
        header.set_row_count(TableId::Assembly, self.assemblies.len() as u32);
        header.set_row_count(TableId::AssemblyRef, self.assembly_refs.len() as u32);
        header.set_row_count(TableId::NestedClass, self.nested_classes.len() as u32);
        header.set_row_count(TableId::GenericParam, self.generic_params.len() as u32);
        header.set_row_count(TableId::MethodSpec, self.method_specs.len() as u32);
        header.set_row_count(
            TableId::GenericParamConstraint,
            self.generic_param_constraints.len() as u32,
        );

        header.write_to(writer);

        let ctx = header.context();

        // Write all table rows in order by TableId
        // 0x00 Module
        for row in &self.modules {
            row.write(writer, &ctx);
        }
        // 0x01 TypeRef
        for row in &self.type_refs {
            row.write(writer, &ctx);
        }
        // 0x02 TypeDef
        for row in &self.type_defs {
            row.write(writer, &ctx);
        }
        // 0x03 FieldPtr
        for row in &self.field_ptrs {
            row.write(writer, &ctx);
        }
        // 0x04 Field
        for row in &self.fields {
            row.write(writer, &ctx);
        }
        // 0x05 MethodPtr
        for row in &self.method_ptrs {
            row.write(writer, &ctx);
        }
        // 0x06 MethodDef
        for row in &self.method_defs {
            row.write(writer, &ctx);
        }
        // 0x07 ParamPtr
        for row in &self.param_ptrs {
            row.write(writer, &ctx);
        }
        // 0x08 Param
        for row in &self.params {
            row.write(writer, &ctx);
        }
        // 0x09 InterfaceImpl
        for row in &self.interface_impls {
            row.write(writer, &ctx);
        }
        // 0x0A MemberRef
        for row in &self.member_refs {
            row.write(writer, &ctx);
        }
        // 0x0B Constant
        for row in &self.constants {
            row.write(writer, &ctx);
        }
        // 0x0C CustomAttribute
        for row in &self.custom_attributes {
            row.write(writer, &ctx);
        }
        // 0x0D FieldMarshal
        for row in &self.field_marshals {
            row.write(writer, &ctx);
        }
        // 0x0E DeclSecurity
        for row in &self.decl_securities {
            row.write(writer, &ctx);
        }
        // 0x0F ClassLayout
        for row in &self.class_layouts {
            row.write(writer, &ctx);
        }
        // 0x10 FieldLayout
        for row in &self.field_layouts {
            row.write(writer, &ctx);
        }
        // 0x11 StandAloneSig
        for row in &self.stand_alone_sigs {
            row.write(writer, &ctx);
        }
        // 0x12 EventMap
        for row in &self.event_maps {
            row.write(writer, &ctx);
        }
        // 0x13 EventPtr
        for row in &self.event_ptrs {
            row.write(writer, &ctx);
        }
        // 0x14 Event
        for row in &self.events {
            row.write(writer, &ctx);
        }
        // 0x15 PropertyMap
        for row in &self.property_maps {
            row.write(writer, &ctx);
        }
        // 0x16 PropertyPtr
        for row in &self.property_ptrs {
            row.write(writer, &ctx);
        }
        // 0x17 Property
        for row in &self.properties {
            row.write(writer, &ctx);
        }
        // 0x18 MethodSemantics
        for row in &self.method_semantics {
            row.write(writer, &ctx);
        }
        // 0x19 MethodImpl
        for row in &self.method_impls {
            row.write(writer, &ctx);
        }
        // 0x1A ModuleRef
        for row in &self.module_refs {
            row.write(writer, &ctx);
        }
        // 0x1B TypeSpec
        for row in &self.type_specs {
            row.write(writer, &ctx);
        }
        // 0x1C ImplMap
        for row in &self.impl_maps {
            row.write(writer, &ctx);
        }
        // 0x1D FieldRva
        for row in &self.field_rvas {
            row.write(writer, &ctx);
        }
        // 0x1E EncLog
        for row in &self.enc_logs {
            row.write(writer, &ctx);
        }
        // 0x1F EncMap
        for row in &self.enc_maps {
            row.write(writer, &ctx);
        }
        // 0x20 Assembly
        for row in &self.assemblies {
            row.write(writer, &ctx);
        }
        // 0x21 AssemblyProcessor (skipped - not parsed)
        // 0x22 AssemblyOs (skipped - not parsed)
        // 0x23 AssemblyRef
        for row in &self.assembly_refs {
            row.write(writer, &ctx);
        }
        // 0x24-0x28 (skipped - not parsed)
        // 0x29 NestedClass
        for row in &self.nested_classes {
            row.write(writer, &ctx);
        }
        // 0x2A GenericParam
        for row in &self.generic_params {
            row.write(writer, &ctx);
        }
        // 0x2B MethodSpec
        for row in &self.method_specs {
            row.write(writer, &ctx);
        }
        // 0x2C GenericParamConstraint
        for row in &self.generic_param_constraints {
            row.write(writer, &ctx);
        }
    }
}
