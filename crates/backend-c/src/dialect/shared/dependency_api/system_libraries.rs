//! Closed local dependency discovery plus original certificate-owned requirements.
use super::CDialect;
use crate::dialect::{CSystemLibrary, file_dependencies};
use portable_codegen::RenderReadyPackage;
use std::collections::BTreeSet;

pub(super) fn collect(
    package: &RenderReadyPackage<CDialect>,
) -> Result<BTreeSet<CSystemLibrary>, String> {
    let unit = package
        .ast()
        .files()
        .iter()
        .flat_map(|file| file.items())
        .next()
        .ok_or("C system-library inventory requires a certified source unit")?;
    let projection = &unit.unit.projection;
    let sources: Vec<_> = projection
        .sources
        .iter()
        .map(|file| file.as_ref().clone())
        .collect();
    let mut libraries = BTreeSet::new();
    for dependency in
        file_dependencies(&projection.registry, &sources).map_err(|error| error.to_string())?
    {
        libraries.extend(dependency.libraries().iter().copied());
    }
    let registry = projection.registry.registrations();
    for (_, imported) in registry.imported_functions() {
        libraries.extend(
            imported
                .package_identity()
                .system_libraries()
                .iter()
                .copied(),
        );
    }
    // A constant-only import still links the original producer translation unit.
    for (_, imported) in registry.imported_constants() {
        libraries.extend(
            imported
                .package_identity()
                .system_libraries()
                .iter()
                .copied(),
        );
    }
    Ok(libraries)
}
