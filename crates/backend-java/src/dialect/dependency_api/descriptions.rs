//! Borrowed source metadata. No constructor, declaration or call authority.
mod collect;
use crate::ast::{JavaDeclaredPath, JavaIdentifier, JavaParameter, JavaType};
pub(super) use collect::collect;
use portable_codegen::{RustDeclarationId, RustSourceOrigin};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaSourceTarget<'a> {
    Declaration(&'a JavaDeclaredPath),
    Field {
        owner: &'a JavaDeclaredPath,
        member: &'a JavaIdentifier,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaSourceDescriptionKind<'a> {
    Function {
        parameters: &'a [JavaParameter],
        result: &'a JavaType,
    },
    Record,
    Field {
        owner: RustDeclarationId,
        ty: &'a JavaType,
    },
}

/// Immutable metadata borrowed from the exact containing owner certificate.
/// Deliberately cannot be converted to a public dependency function.
///
/// ```compile_fail
/// use portable_backend_java::dialect::{JavaSourceDescription, JavaDependencyFunction};
/// fn promote(description: JavaSourceDescription<'_>) -> JavaDependencyFunction {
///     description.into()
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JavaSourceDescription<'a> {
    source: &'a RustSourceOrigin,
    target: JavaSourceTarget<'a>,
    kind: JavaSourceDescriptionKind<'a>,
}
impl<'a> JavaSourceDescription<'a> {
    pub fn source(&self) -> &'a RustSourceOrigin {
        self.source
    }
    pub fn target(&self) -> JavaSourceTarget<'a> {
        self.target
    }
    pub fn kind(&self) -> JavaSourceDescriptionKind<'a> {
        self.kind
    }
}

#[cfg(test)]
#[path = "../../tests/source_descriptions.rs"]
mod tests;
