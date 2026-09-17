//! Reconcile explicit package metadata before canonical projection.
use crate::ast::{CFileRole, CGeneratedOrigin, CRegistry, CSourceFile};

pub(super) fn check(registry: &CRegistry, sources: &[CSourceFile]) -> Result<(), String> {
    let Some(package) = registry.source_package() else {
        return Ok(());
    };
    registry
        .check_file(package.header())
        .map_err(|error| error.to_string())?;
    if sources.len() != 2
        || sources
            .iter()
            .filter(|file| file.identity() == package.header())
            .count()
            != 1
        || sources
            .iter()
            .filter(|file| file.identity().key().role == CFileRole::GeneratedSource)
            .count()
            != 1
    {
        return Err(
            "explicit C source package requires its exact public header/source pair".into(),
        );
    }
    if !package.header().key().path.as_str().ends_with(".h")
        || sources.iter().any(|source| {
            source.identity().key().role == CFileRole::GeneratedSource
                && !source.identity().key().path.as_str().ends_with(".c")
        })
    {
        return Err("explicit C source package requires .h/.c file paths".into());
    }
    for source in sources {
        registry
            .check_file(source.identity())
            .map_err(|error| error.to_string())?;
    }
    for registration in registry.inventory() {
        if let CGeneratedOrigin::RustSource(origin) = &registration.key.origin
            && (origin.declaration.crate_id != package.exports().root.crate_id
                || origin.crate_exports != *package.exports())
        {
            return Err(
                "owned C registration disagrees with explicit source-package provenance".into(),
            );
        }
    }
    Ok(())
}

/// Borrow explicit descriptive provenance from an already certified C package.
/// This cannot create declarations, imports, or dependency authority.
pub fn c_source_package(
    package: &portable_codegen::RenderReadyPackage<super::CDialect>,
) -> Option<&crate::ast::CSourcePackage> {
    package
        .ast()
        .files()
        .iter()
        .flat_map(|file| file.items())
        .next()
        .and_then(|unit| {
            unit.unit
                .projection
                .registry
                .registrations()
                .source_package()
        })
}
