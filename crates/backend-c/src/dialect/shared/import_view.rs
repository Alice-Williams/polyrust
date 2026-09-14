//! Read-only consumer import inventory from a certified package authority.
use super::{CDependencyFunction, CDialect};
use crate::ast::CFunctionRef;
use portable_codegen::RenderReadyPackage;

#[cfg(test)]
#[path = "../../tests/shared_import_view.rs"]
mod tests;

/// A consumer-branded reference paired with its exact owning certificate.
/// Registrations are reported once per package, not once per projected file.
///
/// ```compile_fail
/// use portable_backend_c::dialect::CImportedFunction;
/// fn forge() { let _ = CImportedFunction {}; }
/// ```
pub struct CImportedFunction<'a> {
    function: &'a CFunctionRef,
    dependency: &'a CDependencyFunction,
}

impl<'a> CImportedFunction<'a> {
    pub fn function(&self) -> &'a CFunctionRef {
        self.function
    }

    pub fn dependency(&self) -> &'a CDependencyFunction {
        self.dependency
    }
}

pub fn c_imported_functions(
    package: &RenderReadyPackage<CDialect>,
) -> impl Iterator<Item = CImportedFunction<'_>> {
    // Certification establishes a single shared registry for all units.
    package
        .ast()
        .files()
        .iter()
        .flat_map(|file| file.items())
        .take(1)
        .flat_map(|unit| {
            unit.unit
                .projection
                .registry
                .registrations()
                .imported_functions()
        })
        .map(|(function, dependency)| CImportedFunction {
            function,
            dependency,
        })
}
