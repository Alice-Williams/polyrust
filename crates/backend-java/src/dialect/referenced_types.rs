//! The shared linker has one typed slot for standard and certified foreign types.
use super::JavaImportedResultType;
use crate::ast::JavaKnownType;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaReferencedType {
    Standard(JavaKnownType),
    Dependency(JavaImportedResultType),
}

impl From<JavaKnownType> for JavaReferencedType {
    fn from(value: JavaKnownType) -> Self {
        Self::Standard(value)
    }
}

impl From<JavaImportedResultType> for JavaReferencedType {
    fn from(value: JavaImportedResultType) -> Self {
        Self::Dependency(value)
    }
}
