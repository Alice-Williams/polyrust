//! Mutually exclusive source-crate and canonical-instance package descriptions.
use super::{JavaCanonicalTypePackage, JavaSourcePackage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaPackageMetadata {
    Source(JavaSourcePackage),
    Canonical(Box<JavaCanonicalTypePackage>),
}

impl From<JavaSourcePackage> for JavaPackageMetadata {
    fn from(value: JavaSourcePackage) -> Self {
        Self::Source(value)
    }
}
impl From<JavaCanonicalTypePackage> for JavaPackageMetadata {
    fn from(value: JavaCanonicalTypePackage) -> Self {
        Self::Canonical(value.into())
    }
}

impl JavaPackageMetadata {
    pub fn source(&self) -> Option<&JavaSourcePackage> {
        match self {
            Self::Source(value) => Some(value),
            Self::Canonical(_) => None,
        }
    }
    pub fn canonical(&self) -> Option<&JavaCanonicalTypePackage> {
        match self {
            Self::Canonical(value) => Some(value),
            Self::Source(_) => None,
        }
    }
}
