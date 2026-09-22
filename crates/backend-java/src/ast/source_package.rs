//! Explicit selected-crate description, independent of any owned declaration.
use portable_codegen::{RustCrateExports, RustSourceTypes};
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
    source_types: Option<Arc<RustSourceTypes>>,
}

impl JavaSourcePackage {
    pub fn new(exports: Arc<RustCrateExports>) -> Self {
        Self {
            exports,
            source_types: None,
        }
    }

    pub fn exports(&self) -> &Arc<RustCrateExports> {
        &self.exports
    }

    /// Descriptive compiler facts, reconciled against the exact owner by its API.
    pub fn with_source_types(mut self, source_types: Arc<RustSourceTypes>) -> Self {
        self.source_types = Some(source_types);
        self
    }

    pub fn source_types(&self) -> Option<&Arc<RustSourceTypes>> {
        self.source_types.as_ref()
    }
}
