//! Unnamed standard-library dependencies, derived from typed file metadata.
use super::*;
/// A checked library directive that creates no symbol binding or fake type.
/// Only the linker constructs it; certification reconstructs its exact origin.
///
/// ```compile_fail
/// use portable_codegen::{LinkerDialect, ResolvedLibraryImport};
/// fn forge<D: LinkerDialect>(library: D::StandardLibrary, kind: D::ImportKind)
///     -> ResolvedLibraryImport<D> { ResolvedLibraryImport { library, kind } }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedLibraryImport<D: LinkerDialect> {
    pub(super) library: D::StandardLibrary,
    pub(super) kind: D::ImportKind,
}
impl<D: LinkerDialect> ResolvedLibraryImport<D> {
    pub fn library(&self) -> &D::StandardLibrary {
        &self.library
    }
    pub fn kind(&self) -> &D::ImportKind {
        &self.kind
    }
}
pub(super) fn derive<D: LinkerDialect>(
    dialect: &D,
    source: &TargetFile<D>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ResolvedLibraryImport<D>> {
    let libraries: BTreeSet<_> = dialect
        .file_standard_libraries(source)
        .into_iter()
        .collect();
    libraries
        .into_iter()
        .filter_map(
            |library| match dialect.resolve_standard_library_import(&library) {
                Ok(kind) => Some(ResolvedLibraryImport { library, kind }),
                Err(violation) => {
                    diagnostics.push(Diagnostic::error(
                        violation.code,
                        violation.message,
                        source.source().clone(),
                    ));
                    None
                }
            },
        )
        .collect()
}
pub(super) fn verify<D: LinkerDialect>(
    package: &LinkedTargetPackage<D>,
    file: &LinkedFile<D>,
    source: &TargetFile<D>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if file.library_imports != derive(&package.dialect, source, diagnostics) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InterfaceNonconformance,
            "resolved library imports are not exactly metadata-derived",
            source.source().clone(),
        ));
    }
}
