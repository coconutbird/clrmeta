//! High-level metadata API.

use crate::error::{Error, Result};
use crate::heaps::{BlobHeap, GuidHeap, StringsHeap, UserStringsHeap};
use crate::reader::Reader;
use crate::root::MetadataRoot;
use crate::stream::StreamHeader;
use crate::tables::{
    AssemblyRefRow, AssemblyRow, CustomAttributeRow, FieldRow, MemberRefRow, MethodDefRow,
    ModuleRow, ParamRow, TableContext, TableId, TablesHeader, TypeDefRow, TypeRefRow,
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
    /// Module table rows.
    pub modules: Vec<ModuleRow>,
    /// TypeRef table rows.
    pub type_refs: Vec<TypeRefRow>,
    /// TypeDef table rows.
    pub type_defs: Vec<TypeDefRow>,
    /// Field table rows.
    pub fields: Vec<FieldRow>,
    /// MethodDef table rows.
    pub method_defs: Vec<MethodDefRow>,
    /// Param table rows.
    pub params: Vec<ParamRow>,
    /// MemberRef table rows.
    pub member_refs: Vec<MemberRefRow>,
    /// CustomAttribute table rows.
    pub custom_attributes: Vec<CustomAttributeRow>,
    /// Assembly table rows (usually 0 or 1).
    pub assemblies: Vec<AssemblyRow>,
    /// AssemblyRef table rows.
    pub assembly_refs: Vec<AssemblyRefRow>,
    /// Raw table data for tables we don't parse yet.
    raw_tables_data: Vec<u8>,
}

impl Metadata {
    /// Parse metadata from raw bytes.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let root = MetadataRoot::parse(data)?;

        // Parse heaps
        let strings = Self::parse_heap(&root, data, StreamHeader::STRINGS, StringsHeap::parse)?;
        let user_strings =
            Self::parse_heap(&root, data, StreamHeader::USER_STRINGS, UserStringsHeap::parse)?;
        let guids = Self::parse_heap(&root, data, StreamHeader::GUID, GuidHeap::parse)?;
        let blobs = Self::parse_heap(&root, data, StreamHeader::BLOB, BlobHeap::parse)?;

        // Parse tables stream
        let tables_stream = root
            .tables_stream()
            .ok_or_else(|| Error::StreamNotFound(StreamHeader::TABLES.to_string()))?;
        let tables_data = &data[tables_stream.offset as usize
            ..(tables_stream.offset + tables_stream.size) as usize];
        let mut reader = Reader::new(tables_data);
        let tables_header = TablesHeader::parse(&mut reader)?;
        let ctx = tables_header.context();

        // Parse table rows
        let modules = Self::parse_table(&mut reader, &ctx, TableId::Module, ModuleRow::parse)?;
        let type_refs = Self::parse_table(&mut reader, &ctx, TableId::TypeRef, TypeRefRow::parse)?;
        let type_defs = Self::parse_table(&mut reader, &ctx, TableId::TypeDef, TypeDefRow::parse)?;
        // Skip FieldPtr (0x03) - usually not present
        Self::skip_table(&mut reader, &ctx, TableId::FieldPtr)?;
        let fields = Self::parse_table(&mut reader, &ctx, TableId::Field, FieldRow::parse)?;
        // Skip MethodPtr (0x05) - usually not present
        Self::skip_table(&mut reader, &ctx, TableId::MethodPtr)?;
        let method_defs =
            Self::parse_table(&mut reader, &ctx, TableId::MethodDef, MethodDefRow::parse)?;
        // Skip ParamPtr (0x07) - usually not present
        Self::skip_table(&mut reader, &ctx, TableId::ParamPtr)?;
        let params = Self::parse_table(&mut reader, &ctx, TableId::Param, ParamRow::parse)?;

        // Skip tables 0x09-0x0C temporarily, then parse what we need
        Self::skip_table(&mut reader, &ctx, TableId::InterfaceImpl)?;
        let member_refs =
            Self::parse_table(&mut reader, &ctx, TableId::MemberRef, MemberRefRow::parse)?;
        Self::skip_table(&mut reader, &ctx, TableId::Constant)?;
        let custom_attributes =
            Self::parse_table(&mut reader, &ctx, TableId::CustomAttribute, CustomAttributeRow::parse)?;

        // Store remaining raw data for round-trip
        let raw_tables_data = tables_data[reader.position()..].to_vec();

        // Skip to Assembly table (0x20)
        let assembly_offset = Self::calculate_table_offset(&ctx, TableId::Assembly, tables_data)?;
        let mut asm_reader = Reader::new(&tables_data[assembly_offset..]);
        let assemblies =
            Self::parse_table(&mut asm_reader, &ctx, TableId::Assembly, AssemblyRow::parse)?;

        // Skip to AssemblyRef table (0x23)
        let assembly_ref_offset =
            Self::calculate_table_offset(&ctx, TableId::AssemblyRef, tables_data)?;
        let mut asm_ref_reader = Reader::new(&tables_data[assembly_ref_offset..]);
        let assembly_refs = Self::parse_table(
            &mut asm_ref_reader,
            &ctx,
            TableId::AssemblyRef,
            AssemblyRefRow::parse,
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
            fields,
            method_defs,
            params,
            member_refs,
            custom_attributes,
            assemblies,
            assembly_refs,
            raw_tables_data,
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

    fn calculate_table_offset(
        ctx: &TableContext,
        target: TableId,
        _tables_data: &[u8],
    ) -> Result<usize> {
        // Calculate offset by summing sizes of all preceding tables
        let header_size = 24 + ctx.row_counts.iter().filter(|&&c| c > 0).count() * 4;
        let mut offset = header_size;

        for i in 0..(target as u8) {
            if let Ok(table) = TableId::from_u8(i) {
                let count = ctx.row_count(table) as usize;
                let row_size = ctx.row_size(table);
                offset += count * row_size;
            }
        }

        Ok(offset)
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
                    self.strings.get(row.type_namespace).ok().map(|s| s.to_string())
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
                    self.blobs.get(row.public_key_or_token).ok().map(|b| b.to_vec())
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

        // Update row counts
        header.set_row_count(TableId::Module, self.modules.len() as u32);
        header.set_row_count(TableId::TypeRef, self.type_refs.len() as u32);
        header.set_row_count(TableId::TypeDef, self.type_defs.len() as u32);
        header.set_row_count(TableId::Field, self.fields.len() as u32);
        header.set_row_count(TableId::MethodDef, self.method_defs.len() as u32);
        header.set_row_count(TableId::Param, self.params.len() as u32);
        header.set_row_count(TableId::MemberRef, self.member_refs.len() as u32);
        header.set_row_count(TableId::CustomAttribute, self.custom_attributes.len() as u32);
        header.set_row_count(TableId::Assembly, self.assemblies.len() as u32);
        header.set_row_count(TableId::AssemblyRef, self.assembly_refs.len() as u32);

        header.write_to(writer);

        let ctx = header.context();

        // Write table rows in order
        for row in &self.modules {
            row.write(writer, &ctx);
        }
        for row in &self.type_refs {
            row.write(writer, &ctx);
        }
        for row in &self.type_defs {
            row.write(writer, &ctx);
        }
        for row in &self.fields {
            row.write(writer, &ctx);
        }
        for row in &self.method_defs {
            row.write(writer, &ctx);
        }
        for row in &self.params {
            row.write(writer, &ctx);
        }
        for row in &self.member_refs {
            row.write(writer, &ctx);
        }
        for row in &self.custom_attributes {
            row.write(writer, &ctx);
        }

        // Write remaining raw table data (tables we don't parse)
        writer.write_bytes(&self.raw_tables_data);

        // Write assembly tables at correct position
        for row in &self.assemblies {
            row.write(writer, &ctx);
        }
        for row in &self.assembly_refs {
            row.write(writer, &ctx);
        }
    }
}

