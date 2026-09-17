//! Certificate-derived alias metadata retains the defining producer's authority.
use super::JavaDependencyConstant;
use portable_codegen::{RustDeclarationId, RustExportName};

/// A public Rust binding, not a new field owned by its re-exporting facade.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaForeignConstantExport;
/// let forged = JavaForeignConstantExport {};
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaForeignConstantExport {
    module: RustDeclarationId,
    name: RustExportName,
    dependency: JavaDependencyConstant,
}
impl JavaForeignConstantExport {
    pub(super) fn new(
        module: RustDeclarationId,
        name: RustExportName,
        dependency: JavaDependencyConstant,
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
    pub fn dependency(&self) -> &JavaDependencyConstant {
        &self.dependency
    }
}
