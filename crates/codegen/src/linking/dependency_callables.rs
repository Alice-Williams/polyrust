//! Certified generated-package references, distinct from known library APIs.
use super::*;

/// A certified dependency either binds its fixed native import or names an
/// exact qualified symbol. Qualified references allocate no local import name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DependencySpelling<D: LinkerDialect> {
    FixedImport(D::ImportKind),
    Qualified(D::QualifiedName),
}

/// The dialect reconstructs every field from its opaque dependency witness.
/// This record alone is metadata, never authority to mint a certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyCallableSpec<D: LinkerDialect> {
    pub symbol: D::DependencyCallable,
    pub owner: D::DependencyPackage,
    pub name: D::Identifier,
    pub signature: TargetCallableSignature<D>,
    pub spelling: DependencySpelling<D>,
    pub source: SourceRef,
}

pub(super) fn verify<D: LinkerDialect>(
    catalogue: &SymbolCatalogue<D>,
    dialect: &D,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_unique(
        diagnostics,
        catalogue
            .dependency_callables
            .iter()
            .map(|spec| (&spec.symbol, &spec.source)),
        "certified dependency callable",
    );
    let mut fixed_names = BTreeSet::new();
    let mut qualified_names = BTreeSet::new();
    for spec in &catalogue.dependency_callables {
        let unique = match &spec.spelling {
            DependencySpelling::FixedImport(_) => {
                fixed_names.insert(dialect.identifier_key(&spec.name))
            }
            DependencySpelling::Qualified(name) => qualified_names.insert(name.clone()),
        };
        if !unique {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                "certified dependency native symbols collide",
                spec.source.clone(),
            ));
        }
        if &dialect.dependency_callable_spec(&spec.symbol) != spec {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                "dependency callable metadata differs from its typed witness",
                spec.source.clone(),
            ));
        }
        for error in dialect.verify_signature(&spec.signature) {
            diagnostics.push(Diagnostic::error(
                error.code,
                error.message,
                spec.source.clone(),
            ));
        }
    }
}

pub(super) fn plan<D: LinkerDialect>(
    dialect: &D,
    catalogue: &SymbolCatalogue<D>,
    symbol: &D::DependencyCallable,
) -> Option<ReferencePlan<D>> {
    catalogue
        .dependency_callables
        .iter()
        .find(|spec| &spec.symbol == symbol)
        .map(|spec| {
            let (policy, qualified_name) = match &spec.spelling {
                DependencySpelling::FixedImport(kind) => {
                    (DependencyPolicy::FixedImport(kind.clone()), None)
                }
                DependencySpelling::Qualified(name) => {
                    (DependencyPolicy::Qualified, Some(name.clone()))
                }
            };
            ReferencePlan {
                name: spec.name.clone(),
                alias_stem: String::new(),
                namespace: dialect.callable_namespace(),
                qualified_name,
                origin: SymbolOrigin::CertifiedDependency(spec.owner.clone()),
                policy,
                dependency: None,
            }
        })
}
