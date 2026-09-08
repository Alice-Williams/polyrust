//! Structural C object/declarator types, independent of source rendering.

use std::num::NonZeroU64;

use super::CKnownObject;
use super::registry::{CEnumRef, CStructRef, CTypedefRef, CUnionRef};
use super::signatures::CFunctionType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CScalarType {
    Bool,
    PlainChar,
    Int,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    Size,
    F64,
}

impl CScalarType {
    pub const ALL: [Self; 13] = [
        Self::Bool,
        Self::PlainChar,
        Self::Int,
        Self::I8,
        Self::U8,
        Self::I16,
        Self::U16,
        Self::I32,
        Self::U32,
        Self::I64,
        Self::U64,
        Self::Size,
        Self::F64,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum CConstness {
    #[default]
    Unqualified,
    Const,
}

/// A nonzero constant bound, not a proof of target object-size capacity.
///
/// ```compile_fail
/// use portable_backend_c::ast::CArrayLength;
/// let forged = CArrayLength(std::num::NonZeroU64::new(1).unwrap());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CArrayLength(NonZeroU64);

impl CArrayLength {
    pub fn new(value: u64) -> Result<Self, CTypeError> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or(CTypeError::ZeroArrayLength)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CPointerTarget {
    Void(CConstness),
    Object(Box<CObjectType>),
    Function(Box<CFunctionType>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CObjectTypeKind {
    Scalar(CScalarType),
    Known(CKnownObject),
    Struct(CStructRef),
    Union(CUnionRef),
    Enum(CEnumRef),
    Typedef(CTypedefRef),
    Pointer(CPointerTarget),
    Array {
        element: Box<CObjectType>,
        length: CArrayLength,
    },
}

/// An object type cannot contain a bare void or bare function type.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CObjectType, CReturnType};
/// let invalid = CObjectType::scalar(CReturnType::Void);
/// ```
///
/// Function pointers use an explicit function-target category:
///
/// ```compile_fail
/// use portable_backend_c::ast::{CFunctionType, CObjectType, CReturnType};
/// let function = CFunctionType::new(CReturnType::Void, vec![]);
/// let invalid = CObjectType::pointer(function);
/// ```
///
/// Callers cannot bypass the object's qualifier/shape constructors:
///
/// ```compile_fail
/// use portable_backend_c::ast::{CConstness, CObjectType, CObjectTypeKind, CScalarType};
/// let forged = CObjectType {
///     kind: CObjectTypeKind::Scalar(CScalarType::I32),
///     constness: CConstness::Const,
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CObjectType {
    kind: CObjectTypeKind,
    constness: CConstness,
}

impl CObjectType {
    pub const fn known(value: CKnownObject) -> Self {
        Self {
            kind: CObjectTypeKind::Known(value),
            constness: CConstness::Unqualified,
        }
    }

    pub fn structure(value: CStructRef) -> Self {
        Self {
            kind: CObjectTypeKind::Struct(value),
            constness: CConstness::Unqualified,
        }
    }

    pub fn union(value: CUnionRef) -> Self {
        Self {
            kind: CObjectTypeKind::Union(value),
            constness: CConstness::Unqualified,
        }
    }

    pub fn enumeration(value: CEnumRef) -> Self {
        Self {
            kind: CObjectTypeKind::Enum(value),
            constness: CConstness::Unqualified,
        }
    }

    pub fn typedef(value: CTypedefRef) -> Self {
        Self {
            kind: CObjectTypeKind::Typedef(value),
            constness: CConstness::Unqualified,
        }
    }

    pub const fn scalar(value: CScalarType) -> Self {
        Self {
            kind: CObjectTypeKind::Scalar(value),
            constness: CConstness::Unqualified,
        }
    }

    pub const fn pointer(target: CPointerTarget) -> Self {
        Self {
            kind: CObjectTypeKind::Pointer(target),
            constness: CConstness::Unqualified,
        }
    }

    pub fn array(element: Self, length: CArrayLength) -> Result<Self, CTypeError> {
        element.require_storable()?;
        Ok(Self {
            kind: CObjectTypeKind::Array {
                element: Box::new(element),
                length,
            },
            constness: CConstness::Unqualified,
        })
    }

    pub fn with_constness(mut self, constness: CConstness) -> Result<Self, CTypeError> {
        if self.is_array() && constness == CConstness::Const {
            return Err(CTypeError::ArrayQualifierMustApplyToElement);
        }
        self.constness = constness;
        Ok(self)
    }

    pub const fn kind(&self) -> &CObjectTypeKind {
        &self.kind
    }

    pub const fn constness(&self) -> CConstness {
        self.constness
    }

    pub fn is_array(&self) -> bool {
        let mut root = self;
        while let CObjectTypeKind::Typedef(alias) = root.kind() {
            root = alias.target();
        }
        matches!(root.kind, CObjectTypeKind::Array { .. })
    }

    pub(super) fn without_top_level_const(mut self) -> Self {
        self.constness = CConstness::Unqualified;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CTypeError {
    ExpectedPointer,
    KnownObjectRequiresPointer(CKnownObject),
    ZeroArrayLength,
    ArrayQualifierMustApplyToElement,
    ArrayParameterRequiresPointer,
    ArrayReturn,
    QualifiedReturn,
}

impl std::fmt::Display for CTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedPointer => f.write_str("C null literal requires an exact pointer type"),
            Self::KnownObjectRequiresPointer(value) => {
                write!(f, "library-owned {value:?} requires a borrowed pointer")
            }
            Self::ZeroArrayLength => f.write_str("C17 fixed arrays require a nonzero bound"),
            Self::ArrayQualifierMustApplyToElement => {
                f.write_str("C17 array qualification belongs to its element type")
            }
            Self::ArrayParameterRequiresPointer => {
                f.write_str("C prototype array parameters require an explicit adjusted pointer")
            }
            Self::ArrayReturn => f.write_str("C functions cannot return arrays"),
            Self::QualifiedReturn => f.write_str("top-level return qualifiers fail strict C lint"),
        }
    }
}

impl std::error::Error for CTypeError {}
