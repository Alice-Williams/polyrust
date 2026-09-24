//! Reconstruct allocation from original AST authority, not linked spellings.
use super::*;

pub(super) fn value_scope<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    value: GeneratedValueId,
) -> Result<BindingScope, AstViolation> {
    match dialect.generated_value_owner(package, value)? {
        Some(owner) if package.generated_type(owner).is_some() => Ok(BindingScope::Type(owner)),
        Some(_) => Err(AstViolation::new(
            DiagnosticCode::InterfaceNonconformance,
            "generated value scope refers to an absent owning type",
        )),
        None => Ok(BindingScope::Package),
    }
}

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
    let owned = bindings
        .iter()
        .filter(|binding| binding.scope == BindingScope::Package)
        .map(|binding| {
            (
                binding.namespace.clone(),
                dialect.identifier_key(&binding.identifier),
            )
        })
        .collect::<BTreeSet<_>>();
    let symbols = catalogue
        .dependency_callables
        .iter()
        .map(|spec| {
            (
                dialect.callable_namespace(),
                &spec.name,
                &spec.spelling,
                &spec.source,
            )
        })
        .chain(catalogue.dependency_values.iter().map(|spec| {
            (
                dialect.value_namespace(),
                &spec.name,
                &spec.spelling,
                &spec.source,
            )
        }));
    for (namespace, name, spelling, source) in symbols {
        if matches!(spelling, DependencySpelling::FixedImport(_))
            && owned.contains(&(namespace, dialect.identifier_key(name)))
        {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                "fixed dependency symbol collides with an owned package binding",
                source.clone(),
            ));
        }
    }
}
