# clrmeta

ECMA-335 CLI/.NET metadata parsing library for Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- Parse BSJB metadata root and stream headers
- Access heaps: #Strings, #US, #GUID, #Blob
- Parse metadata tables: Module, TypeDef, TypeRef, MethodDef, Assembly, AssemblyRef, etc.
- High-level API for common queries (assembly info, types, methods)
- No PE dependency - works with raw metadata bytes

## Usage

```rust
use clrmeta::Metadata;

// Parse metadata from raw bytes (e.g., from PE's CLR data directory)
let metadata = Metadata::parse(&metadata_bytes)?;

println!("Runtime version: {}", metadata.version());

if let Some(assembly) = metadata.assembly() {
    println!("Assembly: {} v{}.{}.{}.{}",
        assembly.name,
        assembly.version.0, assembly.version.1,
        assembly.version.2, assembly.version.3);
}

for type_def in metadata.types() {
    println!("Type: {}", type_def.full_name());
}
```

## Integration with portex

This crate is designed to work with [portex](https://github.com/coconutbird/portex) for parsing .NET assemblies from PE files:

```rust
use portex::PE;
use clrmeta::Metadata;

let pe = PE::from_file("example.exe")?;

if let Some(cli_header) = pe.cli_header()? {
    let metadata_bytes = pe.read_at_rva(
        cli_header.metadata_rva,
        cli_header.metadata_size as usize
    ).unwrap();
    
    let metadata = Metadata::parse(metadata_bytes)?;
    // ...
}
```

## References

- [ECMA-335 CLI Specification](https://www.ecma-international.org/publications-and-standards/standards/ecma-335/)

## License

MIT
