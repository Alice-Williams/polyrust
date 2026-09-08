//! Complete initializer shapes without raw brace/token payloads.

use super::{
    CExpressionError, CMemberRef, CObjectType, CRegistryError, CStructRef, CTypeError, CUnionRef,
    CValue, registry::RegistryScope,
};

/// Callers cannot replace a checked initializer shape with a different one.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CInitializer, CInitializerKind};
/// fn forge(mut value: CInitializer, kind: CInitializerKind) { value.kind = kind; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CInitializer {
    pub(super) brand: RegistryScope,
    pub(super) ty: CObjectType,
    pub(super) kind: CInitializerKind,
}

impl CInitializer {
    pub const fn ty(&self) -> &CObjectType {
        &self.ty
    }
    pub const fn kind(&self) -> &CInitializerKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CInitializerKind {
    Expression(CValue),
    Zero(CObjectType),
    Array {
        declared_type: CObjectType,
        elements: Vec<CInitializer>,
    },
    Struct {
        owner: CStructRef,
        members: Vec<(CMemberRef, CInitializer)>,
    },
    Union {
        owner: CUnionRef,
        member: CMemberRef,
        value: Box<CInitializer>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CInitializerError {
    Expression(CExpressionError),
    Registry(CRegistryError),
    Type(CTypeError),
    ExpectedArray,
    TypeMismatch,
    ElementCount { expected: u64, actual: usize },
    IncompleteAggregate,
    MemberInventoryMismatch,
}

impl From<CExpressionError> for CInitializerError {
    fn from(value: CExpressionError) -> Self {
        Self::Expression(value)
    }
}
impl From<CRegistryError> for CInitializerError {
    fn from(value: CRegistryError) -> Self {
        Self::Registry(value)
    }
}
impl From<CTypeError> for CInitializerError {
    fn from(value: CTypeError) -> Self {
        Self::Type(value)
    }
}
impl std::fmt::Display for CInitializerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expression(value) => value.fmt(f),
            Self::Registry(value) => value.fmt(f),
            Self::Type(value) => value.fmt(f),
            Self::ExpectedArray => f.write_str("C array initializer requires an array target"),
            Self::TypeMismatch => f.write_str("C initializer does not match its destination type"),
            Self::ElementCount { expected, actual } => write!(
                f,
                "C array initializer expects {expected} elements, received {actual}"
            ),
            Self::IncompleteAggregate => {
                f.write_str("C aggregate initializer requires its complete registered definition")
            }
            Self::MemberInventoryMismatch => {
                f.write_str("C struct initializer must match the complete ordered member inventory")
            }
        }
    }
}
impl std::error::Error for CInitializerError {}
