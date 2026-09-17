//! Read-only import inventory from the consumer's certified shared registry.
use super::{CDependencyConstant, CDialect};
use crate::ast::CObjectRef;
use portable_codegen::RenderReadyPackage;

/// Exact consumer object and retained original producer; never a descriptor.
pub struct CImportedConstant<'a> {
    object: &'a CObjectRef,
    dependency: &'a CDependencyConstant,
}
impl<'a> CImportedConstant<'a> {
    pub fn object(&self) -> &'a CObjectRef {
        self.object
    }
    pub fn dependency(&self) -> &'a CDependencyConstant {
        self.dependency
    }
}
pub fn c_imported_constants(
    package: &RenderReadyPackage<CDialect>,
) -> impl Iterator<Item = CImportedConstant<'_>> {
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
                .imported_constants()
        })
        .map(|(object, dependency)| CImportedConstant { object, dependency })
}

/// Imported objects actually referenced by declarations or expressions, excluding
/// source-package export-only roots. All returned witnesses come from this exact
/// render-ready package's frozen registry.
pub fn c_used_imported_constants(
    package: &RenderReadyPackage<CDialect>,
) -> impl Iterator<Item = CImportedConstant<'_>> {
    let objects: std::collections::BTreeSet<_> = package
        .ast()
        .files()
        .iter()
        .flat_map(|file| file.items())
        .flat_map(|unit| crate::dialect::dependencies::object_references(&unit.unit.data.source))
        .collect();
    c_imported_constants(package).filter(move |value| objects.contains(value.object()))
}
