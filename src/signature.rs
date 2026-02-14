//! Signature parsing for method, field, and property signatures.
//!
//! ECMA-335 II.23.2 defines the blob signature format.

use crate::error::{Error, Result};
use crate::reader::Reader;

/// Element type codes (ECMA-335 II.23.1.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ElementType {
    End = 0x00,
    Void = 0x01,
    Boolean = 0x02,
    Char = 0x03,
    I1 = 0x04,
    U1 = 0x05,
    I2 = 0x06,
    U2 = 0x07,
    I4 = 0x08,
    U4 = 0x09,
    I8 = 0x0A,
    U8 = 0x0B,
    R4 = 0x0C,
    R8 = 0x0D,
    String = 0x0E,
    Ptr = 0x0F,
    ByRef = 0x10,
    ValueType = 0x11,
    Class = 0x12,
    Var = 0x13,
    Array = 0x14,
    GenericInst = 0x15,
    TypedByRef = 0x16,
    IntPtr = 0x18,
    UIntPtr = 0x19,
    FnPtr = 0x1B,
    Object = 0x1C,
    SzArray = 0x1D,
    MVar = 0x1E,
    CModReqd = 0x1F,
    CModOpt = 0x20,
    Internal = 0x21,
    Modifier = 0x40,
    Sentinel = 0x41,
    Pinned = 0x45,
}

impl ElementType {
    /// Parse element type from byte.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::End),
            0x01 => Some(Self::Void),
            0x02 => Some(Self::Boolean),
            0x03 => Some(Self::Char),
            0x04 => Some(Self::I1),
            0x05 => Some(Self::U1),
            0x06 => Some(Self::I2),
            0x07 => Some(Self::U2),
            0x08 => Some(Self::I4),
            0x09 => Some(Self::U4),
            0x0A => Some(Self::I8),
            0x0B => Some(Self::U8),
            0x0C => Some(Self::R4),
            0x0D => Some(Self::R8),
            0x0E => Some(Self::String),
            0x0F => Some(Self::Ptr),
            0x10 => Some(Self::ByRef),
            0x11 => Some(Self::ValueType),
            0x12 => Some(Self::Class),
            0x13 => Some(Self::Var),
            0x14 => Some(Self::Array),
            0x15 => Some(Self::GenericInst),
            0x16 => Some(Self::TypedByRef),
            0x18 => Some(Self::IntPtr),
            0x19 => Some(Self::UIntPtr),
            0x1B => Some(Self::FnPtr),
            0x1C => Some(Self::Object),
            0x1D => Some(Self::SzArray),
            0x1E => Some(Self::MVar),
            0x1F => Some(Self::CModReqd),
            0x20 => Some(Self::CModOpt),
            0x21 => Some(Self::Internal),
            0x40 => Some(Self::Modifier),
            0x41 => Some(Self::Sentinel),
            0x45 => Some(Self::Pinned),
            _ => None,
        }
    }

    /// Get a human-readable name for the element type.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::End => "end",
            Self::Void => "void",
            Self::Boolean => "bool",
            Self::Char => "char",
            Self::I1 => "sbyte",
            Self::U1 => "byte",
            Self::I2 => "short",
            Self::U2 => "ushort",
            Self::I4 => "int",
            Self::U4 => "uint",
            Self::I8 => "long",
            Self::U8 => "ulong",
            Self::R4 => "float",
            Self::R8 => "double",
            Self::String => "string",
            Self::Ptr => "ptr",
            Self::ByRef => "byref",
            Self::ValueType => "valuetype",
            Self::Class => "class",
            Self::Var => "!T",
            Self::Array => "array",
            Self::GenericInst => "generic",
            Self::TypedByRef => "typedref",
            Self::IntPtr => "nint",
            Self::UIntPtr => "nuint",
            Self::FnPtr => "fnptr",
            Self::Object => "object",
            Self::SzArray => "[]",
            Self::MVar => "!!T",
            Self::CModReqd => "modreq",
            Self::CModOpt => "modopt",
            Self::Internal => "internal",
            Self::Modifier => "modifier",
            Self::Sentinel => "...",
            Self::Pinned => "pinned",
        }
    }
}

/// Calling convention flags (ECMA-335 II.23.2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallingConvention(pub u8);

impl CallingConvention {
    pub const DEFAULT: u8 = 0x00;
    pub const VARARG: u8 = 0x05;
    pub const FIELD: u8 = 0x06;
    pub const LOCAL_SIG: u8 = 0x07;
    pub const PROPERTY: u8 = 0x08;
    pub const GENERIC: u8 = 0x10;
    pub const HAS_THIS: u8 = 0x20;
    pub const EXPLICIT_THIS: u8 = 0x40;

    /// Check if this is a method signature.
    #[must_use]
    pub fn is_method(self) -> bool {
        let base = self.0 & 0x0F;
        base == Self::DEFAULT || base == Self::VARARG
    }

    /// Check if this is a field signature.
    #[must_use]
    pub fn is_field(self) -> bool {
        (self.0 & 0x0F) == Self::FIELD
    }

    /// Check if this is a property signature.
    #[must_use]
    pub fn is_property(self) -> bool {
        (self.0 & 0x0F) == Self::PROPERTY
    }

    /// Check if the method has an instance pointer (this).
    #[must_use]
    pub fn has_this(self) -> bool {
        (self.0 & Self::HAS_THIS) != 0
    }

    /// Check if this is a generic method.
    #[must_use]
    pub fn is_generic(self) -> bool {
        (self.0 & Self::GENERIC) != 0
    }
}

/// A parsed type from a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSig {
    /// Primitive type (void, bool, char, i1, u1, etc.).
    Primitive(ElementType),
    /// Class or interface reference (TypeDefOrRef coded index).
    Class(u32),
    /// Value type reference (TypeDefOrRef coded index).
    ValueType(u32),
    /// Single-dimensional zero-based array.
    SzArray(Box<TypeSig>),
    /// Multi-dimensional array with bounds.
    Array {
        element_type: Box<TypeSig>,
        rank: u32,
        sizes: Vec<u32>,
        lo_bounds: Vec<i32>,
    },
    /// Pointer to type.
    Ptr(Box<TypeSig>),
    /// By-reference parameter.
    ByRef(Box<TypeSig>),
    /// Generic type instantiation.
    GenericInst {
        is_value_type: bool,
        type_ref: u32,
        type_args: Vec<TypeSig>,
    },
    /// Generic type parameter (T).
    Var(u32),
    /// Generic method parameter (TMethod).
    MVar(u32),
    /// Function pointer.
    FnPtr(Box<MethodSig>),
    /// Modified type (modreq/modopt).
    Modified {
        required: bool,
        modifier: u32,
        inner: Box<TypeSig>,
    },
    /// Pinned type (for locals).
    Pinned(Box<TypeSig>),
}

impl TypeSig {
    /// Parse a type from a signature blob.
    pub fn parse(reader: &mut Reader<'_>) -> Result<Self> {
        let elem = reader.read_u8()?;

        match elem {
            // Primitives
            0x01 => Ok(TypeSig::Primitive(ElementType::Void)),
            0x02 => Ok(TypeSig::Primitive(ElementType::Boolean)),
            0x03 => Ok(TypeSig::Primitive(ElementType::Char)),
            0x04 => Ok(TypeSig::Primitive(ElementType::I1)),
            0x05 => Ok(TypeSig::Primitive(ElementType::U1)),
            0x06 => Ok(TypeSig::Primitive(ElementType::I2)),
            0x07 => Ok(TypeSig::Primitive(ElementType::U2)),
            0x08 => Ok(TypeSig::Primitive(ElementType::I4)),
            0x09 => Ok(TypeSig::Primitive(ElementType::U4)),
            0x0A => Ok(TypeSig::Primitive(ElementType::I8)),
            0x0B => Ok(TypeSig::Primitive(ElementType::U8)),
            0x0C => Ok(TypeSig::Primitive(ElementType::R4)),
            0x0D => Ok(TypeSig::Primitive(ElementType::R8)),
            0x0E => Ok(TypeSig::Primitive(ElementType::String)),
            0x16 => Ok(TypeSig::Primitive(ElementType::TypedByRef)),
            0x18 => Ok(TypeSig::Primitive(ElementType::IntPtr)),
            0x19 => Ok(TypeSig::Primitive(ElementType::UIntPtr)),
            0x1C => Ok(TypeSig::Primitive(ElementType::Object)),

            // Class
            0x12 => {
                let token = reader.read_compressed_uint()?;
                Ok(TypeSig::Class(token))
            }

            // ValueType
            0x11 => {
                let token = reader.read_compressed_uint()?;
                Ok(TypeSig::ValueType(token))
            }

            // SzArray
            0x1D => {
                let elem_type = TypeSig::parse(reader)?;
                Ok(TypeSig::SzArray(Box::new(elem_type)))
            }

            // Array
            0x14 => {
                let elem_type = TypeSig::parse(reader)?;
                let rank = reader.read_compressed_uint()?;
                let num_sizes = reader.read_compressed_uint()?;
                let mut sizes = Vec::with_capacity(num_sizes as usize);
                for _ in 0..num_sizes {
                    sizes.push(reader.read_compressed_uint()?);
                }
                let num_lo_bounds = reader.read_compressed_uint()?;
                let mut lo_bounds = Vec::with_capacity(num_lo_bounds as usize);
                for _ in 0..num_lo_bounds {
                    lo_bounds.push(reader.read_compressed_int()?);
                }
                Ok(TypeSig::Array {
                    element_type: Box::new(elem_type),
                    rank,
                    sizes,
                    lo_bounds,
                })
            }

            // Ptr
            0x0F => {
                let inner = TypeSig::parse(reader)?;
                Ok(TypeSig::Ptr(Box::new(inner)))
            }

            // ByRef
            0x10 => {
                let inner = TypeSig::parse(reader)?;
                Ok(TypeSig::ByRef(Box::new(inner)))
            }

            // GenericInst
            0x15 => {
                let is_value_type = reader.read_u8()? == 0x11;
                let type_ref = reader.read_compressed_uint()?;
                let gen_arg_count = reader.read_compressed_uint()?;
                let mut type_args = Vec::with_capacity(gen_arg_count as usize);
                for _ in 0..gen_arg_count {
                    type_args.push(TypeSig::parse(reader)?);
                }
                Ok(TypeSig::GenericInst {
                    is_value_type,
                    type_ref,
                    type_args,
                })
            }

            // Var (generic type param)
            0x13 => {
                let index = reader.read_compressed_uint()?;
                Ok(TypeSig::Var(index))
            }

            // MVar (generic method param)
            0x1E => {
                let index = reader.read_compressed_uint()?;
                Ok(TypeSig::MVar(index))
            }

            // FnPtr
            0x1B => {
                let method_sig = MethodSig::parse(reader)?;
                Ok(TypeSig::FnPtr(Box::new(method_sig)))
            }

            // CModReqd
            0x1F => {
                let modifier = reader.read_compressed_uint()?;
                let inner = TypeSig::parse(reader)?;
                Ok(TypeSig::Modified {
                    required: true,
                    modifier,
                    inner: Box::new(inner),
                })
            }

            // CModOpt
            0x20 => {
                let modifier = reader.read_compressed_uint()?;
                let inner = TypeSig::parse(reader)?;
                Ok(TypeSig::Modified {
                    required: false,
                    modifier,
                    inner: Box::new(inner),
                })
            }

            // Pinned
            0x45 => {
                let inner = TypeSig::parse(reader)?;
                Ok(TypeSig::Pinned(Box::new(inner)))
            }

            _ => Err(Error::InvalidBlob(reader.position())),
        }
    }
}

/// A parsed method signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSig {
    /// Calling convention flags.
    pub calling_convention: CallingConvention,
    /// Number of generic parameters (0 if not generic).
    pub generic_param_count: u32,
    /// Return type.
    pub return_type: TypeSig,
    /// Parameter types.
    pub params: Vec<TypeSig>,
    /// Sentinel index for vararg methods (None if not vararg).
    pub sentinel: Option<usize>,
}

impl MethodSig {
    /// Parse a method signature from a blob.
    pub fn parse(reader: &mut Reader<'_>) -> Result<Self> {
        let cc = reader.read_u8()?;
        let calling_convention = CallingConvention(cc);

        let generic_param_count = if (cc & CallingConvention::GENERIC) != 0 {
            reader.read_compressed_uint()?
        } else {
            0
        };

        let param_count = reader.read_compressed_uint()?;
        let return_type = TypeSig::parse(reader)?;

        let mut params = Vec::with_capacity(param_count as usize);
        let mut sentinel = None;

        for i in 0..param_count as usize {
            // Check for sentinel (vararg boundary)
            if reader.remaining() > 0 {
                let peek = reader.peek_u8()?;
                if peek == 0x41 {
                    reader.read_u8()?; // consume sentinel
                    sentinel = Some(i);
                }
            }
            params.push(TypeSig::parse(reader)?);
        }

        Ok(Self {
            calling_convention,
            generic_param_count,
            return_type,
            params,
            sentinel,
        })
    }

    /// Parse a method signature from raw bytes.
    pub fn parse_blob(data: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(data);
        Self::parse(&mut reader)
    }
}

/// A parsed field signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSig {
    /// The field type.
    pub field_type: TypeSig,
}

impl FieldSig {
    /// Parse a field signature from a blob.
    pub fn parse(reader: &mut Reader<'_>) -> Result<Self> {
        let cc = reader.read_u8()?;
        if cc != CallingConvention::FIELD {
            return Err(Error::InvalidBlob(reader.position()));
        }
        let field_type = TypeSig::parse(reader)?;
        Ok(Self { field_type })
    }

    /// Parse a field signature from raw bytes.
    pub fn parse_blob(data: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(data);
        Self::parse(&mut reader)
    }
}

/// A parsed property signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertySig {
    /// Whether this is an instance property (has this).
    pub has_this: bool,
    /// Property type.
    pub property_type: TypeSig,
    /// Parameter types (for indexed properties).
    pub params: Vec<TypeSig>,
}

impl PropertySig {
    /// Parse a property signature from a blob.
    pub fn parse(reader: &mut Reader<'_>) -> Result<Self> {
        let cc = reader.read_u8()?;
        if (cc & 0x0F) != CallingConvention::PROPERTY {
            return Err(Error::InvalidBlob(reader.position()));
        }
        let has_this = (cc & CallingConvention::HAS_THIS) != 0;

        let param_count = reader.read_compressed_uint()?;
        let property_type = TypeSig::parse(reader)?;

        let mut params = Vec::with_capacity(param_count as usize);
        for _ in 0..param_count {
            params.push(TypeSig::parse(reader)?);
        }

        Ok(Self {
            has_this,
            property_type,
            params,
        })
    }

    /// Parse a property signature from raw bytes.
    pub fn parse_blob(data: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(data);
        Self::parse(&mut reader)
    }
}

/// A parsed local variables signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVarSig {
    /// Local variable types.
    pub locals: Vec<TypeSig>,
}

impl LocalVarSig {
    /// Parse a local variables signature from a blob.
    pub fn parse(reader: &mut Reader<'_>) -> Result<Self> {
        let cc = reader.read_u8()?;
        if cc != CallingConvention::LOCAL_SIG {
            return Err(Error::InvalidBlob(reader.position()));
        }

        let count = reader.read_compressed_uint()?;
        let mut locals = Vec::with_capacity(count as usize);

        for _ in 0..count {
            locals.push(TypeSig::parse(reader)?);
        }

        Ok(Self { locals })
    }

    /// Parse a local variables signature from raw bytes.
    pub fn parse_blob(data: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(data);
        Self::parse(&mut reader)
    }
}

