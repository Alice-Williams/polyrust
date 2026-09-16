//! Explicit selected-crate description, independent of any owned declaration.
use portable_codegen::RustCrateExports;
use std::sync::Arc;

/// Unresolved metadata, not evidence of compiler acceptance or renderability.
/// Certification reconciles this graph with the exact facade and source origins.
///
/// ```compile_fail
/// use portable_backend_java::ast::JavaSourcePackage;
/// let unchecked = JavaSourcePackage { exports: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaSourcePackage {
    exports: Arc<RustCrateExports>,
}

impl JavaSourcePackage {
    pub fn new(exports: Arc<RustCrateExports>) -> Self {
        Self { exports }
    }

    pub fn exports(&self) -> &Arc<RustCrateExports> {
        &self.exports
    }
}
