//! Reconstruct reference membership from original roots, never linked claims.
use super::{
    LinkedTargetPackage, LinkerDialect, collect_references, derive_and_validate_file_graph,
    expand_file_helpers, file_imports, library_imports,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};

pub(super) fn verify<D: LinkerDialect>(
    package: &LinkedTargetPackage<D>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut expected = collect_references(&package.dialect, &package.unresolved, diagnostics);
    expand_file_helpers(
        &package.dialect,
        &package.unresolved,
        &package.catalogue,
        &mut expected,
        diagnostics,
    );
    derive_and_validate_file_graph(
        &package.dialect,
        &package.unresolved,
        &mut expected,
        diagnostics,
    );
    for expected in expected {
        let Some(actual) = package.files.iter().find(|file| file.file == expected.file) else {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnresolvedReference,
                "canonical source-file reference inventory has no linked file",
                SourceRef::logical(["target-linker", "reference-inventory"]),
            ));
            continue;
        };
        // Dependencies may exist solely for reachability/visibility and need
        // not consume a name during dialect item resolution. Thus reconstruct
        // even the references which a renderer never spells.
        if !actual
            .references
            .iter()
            .map(|reference| (&reference.symbol, &reference.source))
            .eq(expected
                .references
                .iter()
                .map(|reference| (&reference.symbol, &reference.source)))
        {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                "linked reference inventory is not exactly unresolved-root-derived",
                actual.source.clone(),
            ));
        }
        if actual.dependencies != expected.dependencies {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                "file dependency inventory is not exactly unresolved-root-derived",
                actual.source.clone(),
            ));
        }
        let source = package
            .unresolved
            .file(expected.file)
            .expect("canonical inventory was collected from this unresolved package");
        file_imports::verify(package, actual, source, expected.dependencies, diagnostics);
        library_imports::verify(package, actual, source, diagnostics);
    }
}
