//! Exact-width literal payloads; no source tokens or floating-bit pseudo-AST.

use super::{CObjectType, CObjectTypeKind, CScalarType, CTypeError};

/// Rust checks the payload width before the AST can be constructed.
///
/// ```compile_fail
/// use portable_backend_c::ast::CSignedLiteral;
/// fn invalid(value: i32) { let _ = CSignedLiteral::I8(value); }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CSignedLiteral {
    PlainChar(i8),
    Int(i32),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

impl CSignedLiteral {
    pub const fn scalar_type(self) -> CScalarType {
        match self {
            Self::PlainChar(_) => CScalarType::PlainChar,
            Self::Int(_) => CScalarType::Int,
            Self::I8(_) => CScalarType::I8,
            Self::I16(_) => CScalarType::I16,
            Self::I32(_) => CScalarType::I32,
            Self::I64(_) => CScalarType::I64,
        }
    }
}

/// ```compile_fail
/// use portable_backend_c::ast::CUnsignedLiteral;
/// fn invalid(value: u64) { let _ = CUnsignedLiteral::U8(value); }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CUnsignedLiteral {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Size(u64),
}

impl CUnsignedLiteral {
    pub const fn scalar_type(self) -> CScalarType {
        match self {
            Self::U8(_) => CScalarType::U8,
            Self::U16(_) => CScalarType::U16,
            Self::U32(_) => CScalarType::U32,
            Self::U64(_) => CScalarType::U64,
            Self::Size(_) => CScalarType::Size,
        }
    }
}

/// Pointer category witness only: not proof of initialization or ownership.
/// Retains the declared type so alias authentication cannot be bypassed.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CNullPointer, CObjectType};
/// fn forge(ty: CObjectType) { let _ = CNullPointer { declared: ty.clone(), ty }; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CNullPointer {
    declared: CObjectType,
    ty: CObjectType,
}

impl CNullPointer {
    pub fn new(declared: CObjectType) -> Result<Self, CTypeError> {
        let ty = declared.canonical().without_top_level_const();
        if !matches!(ty.kind(), CObjectTypeKind::Pointer(_)) {
            return Err(CTypeError::ExpectedPointer);
        }
        Ok(Self { declared, ty })
    }

    pub const fn ty(&self) -> &CObjectType {
        &self.ty
    }

    pub const fn declared_type(&self) -> &CObjectType {
        &self.declared
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CLiteral {
    Bool(bool),
    Signed(CSignedLiteral),
    Unsigned(CUnsignedLiteral),
    CharByte(u8),
    NullPointer(CNullPointer),
}

impl CLiteral {
    pub fn ty(&self) -> CObjectType {
        let scalar = match self {
            Self::Bool(_) => CScalarType::Bool,
            Self::Signed(value) => value.scalar_type(),
            Self::Unsigned(value) => value.scalar_type(),
            Self::CharByte(_) => CScalarType::U8,
            Self::NullPointer(value) => return value.ty().clone(),
        };
        CObjectType::scalar(scalar)
    }
}
