//! Reconstruct allocation from original AST authority, not linked spellings.
use super::*;

pub(super) fn verify<D: LinkerDialect>(
    package: &LinkedTargetPackage<D>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut roots = collect_references(&package.dialect, &package.unresolved, diagnostics);
    let helpers = expand_file_helpers(
        &package.dialect,
        &package.unresolved,
        &package.catalogue,
        &mut roots,
        diagnostics,
    );
    let helper_ids = helpers.iter().map(|helper| helper.id.clone()).collect();
    let expected = allocate_bindings(
        &package.dialect,
        &package.unresolved,
        &package.catalogue,
        &helper_ids,
        diagnostics,
    );
    if expected != package.bindings {
        diagnostics.push(link_error(
            DiagnosticCode::InterfaceNonconformance,
            "linked bindings are not exactly original-package-derived",
            "bindings",
        ));
    }
    verify_dependency_names(
        &package.dialect,
        &package.catalogue,
        &package.bindings,
        diagnostics,
    );
}

pub(super) fn verify_dependency_names<D: LinkerDialect>(
    dialect: &D,
    catalogue: &SymbolCatalogue<D>,
    bindings: &[ResolvedBinding<D>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let namespace = dialect.callable_namespace();
    let owned = bindings
        .iter()
        .filter(|binding| binding.scope == BindingScope::Package && binding.namespace == namespace)
        .map(|binding| dialect.identifier_key(&binding.identifier))
        .collect::<BTreeSet<_>>();
    for spec in &catalogue.dependency_callables {
        if matches!(spec.spelling, DependencySpelling::FixedImport(_))
            && owned.contains(&dialect.identifier_key(&spec.name))
        {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                "fixed dependency symbol collides with an owned package binding",
                spec.source.clone(),
            ));
        }
    }
}
