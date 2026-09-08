//! Unresolved file grouping. Includes/guards are added only by the linker.

use super::{CAssertDiagnostic, CComment, CDeclaration, CDefinition, CFileRef, CValue};

/// This file has no rendering API and is not a source validity certificate.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CFileItem, CSourceFile};
/// fn inject(mut file: CSourceFile, item: CFileItem) { file.items.push(item); }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSourceFile {
    pub(super) file: CFileRef,
    pub(super) items: Vec<CFileItem>,
}
impl CSourceFile {
    pub const fn identity(&self) -> &CFileRef {
        &self.file
    }
    pub fn items(&self) -> &[CFileItem] {
        &self.items
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CFileItem {
    Declaration(CDeclaration),
    Definition(CDefinition),
    Comment(CComment),
    StaticAssert(CStaticAssertion),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CStaticAssertion {
    pub(super) file: CFileRef,
    pub(super) condition: CValue,
    pub(super) diagnostic: CAssertDiagnostic,
}
impl CStaticAssertion {
    pub const fn file(&self) -> &CFileRef {
        &self.file
    }
    pub const fn condition(&self) -> &CValue {
        &self.condition
    }
    pub const fn diagnostic(&self) -> &CAssertDiagnostic {
        &self.diagnostic
    }
}
