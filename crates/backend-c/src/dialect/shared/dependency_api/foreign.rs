//! Read-only source alias metadata backed by the original producer certificate.
use super::CDependencyConstant;
use portable_codegen::{RustDeclarationId, RustExportName};

/// A certified facade binding; it does not transfer ownership of the constant.
///
/// ```compile_fail
/// use portable_backend_c::dialect::CForeignConstantExport;
/// fn forge() { let _ = CForeignConstantExport {}; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CForeignConstantExport {
    module: RustDeclarationId,
    name: RustExportName,
    dependency: CDependencyConstant,
}

impl CForeignConstantExport {
    pub(super) fn new(
        module: RustDeclarationId,
        name: RustExportName,
        dependency: CDependencyConstant,
    ) -> Self {
        Self {
            module,
            name,
            dependency,
        }
    }

    pub fn module(&self) -> RustDeclarationId {
        self.module
    }
    pub fn name(&self) -> &RustExportName {
        &self.name
    }
    pub fn dependency(&self) -> &CDependencyConstant {
        &self.dependency
    }
}
