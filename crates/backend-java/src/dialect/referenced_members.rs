//! Standard and original-certificate members share the structural linker.
use super::{
    JavaImportedResultAccessor, JavaImportedResultConstructor, JavaKnownConstructor,
    JavaKnownMethod,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaReferencedMemberName {
    Standard(super::JavaMemberName),
    Dependency(JavaImportedResultAccessor),
}
impl JavaReferencedMemberName {
    pub fn text(&self) -> &str {
        match self {
            Self::Standard(value) => value.text(),
            Self::Dependency(value) => value.name().as_str(),
        }
    }
}
impl From<super::JavaMemberName> for JavaReferencedMemberName {
    fn from(value: super::JavaMemberName) -> Self {
        Self::Standard(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaReferencedConstructor {
    Standard(JavaKnownConstructor),
    Dependency(JavaImportedResultConstructor),
}
impl From<JavaKnownConstructor> for JavaReferencedConstructor {
    fn from(value: JavaKnownConstructor) -> Self {
        Self::Standard(value)
    }
}
impl From<JavaImportedResultConstructor> for JavaReferencedConstructor {
    fn from(value: JavaImportedResultConstructor) -> Self {
        Self::Dependency(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaReferencedMethod {
    Standard(JavaKnownMethod),
    Dependency(JavaImportedResultAccessor),
}
impl From<JavaKnownMethod> for JavaReferencedMethod {
    fn from(value: JavaKnownMethod) -> Self {
        Self::Standard(value)
    }
}
impl From<JavaImportedResultAccessor> for JavaReferencedMethod {
    fn from(value: JavaImportedResultAccessor) -> Self {
        Self::Dependency(value)
    }
}
