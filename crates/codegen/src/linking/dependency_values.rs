//! Certified context-free value references retain their exact producer authority.
use super::*;

/// Explicit absence of a dependency-value implementation, never a success flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NoDependencyValue {}

/// Reconstructed metadata; only the dialect's opaque witness carries authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyValueSpec<D: LinkerDialect> {
    pub symbol: D::DependencyValue,
    pub owner: D::DependencyPackage,
    pub name: D::Identifier,
    pub ty: TargetTypeRef<D>,
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
            .dependency_values
            .iter()
            .map(|spec| (&spec.symbol, &spec.source)),
        "certified dependency value",
    );
    let mut fixed = BTreeSet::new();
    let mut qualified = BTreeSet::new();
    for spec in &catalogue.dependency_callables {
        match &spec.spelling {
            DependencySpelling::FixedImport(_) => {
                fixed.insert((
                    dialect.callable_namespace(),
                    dialect.identifier_key(&spec.name),
                ));
            }
            DependencySpelling::Qualified(name) => {
                qualified.insert((dialect.callable_namespace(), name.clone()));
            }
        }
    }
    for spec in &catalogue.dependency_values {
        let unique = match &spec.spelling {
            DependencySpelling::FixedImport(_) => fixed.insert((
                dialect.value_namespace(),
                dialect.identifier_key(&spec.name),
            )),
            DependencySpelling::Qualified(name) => {
                qualified.insert((dialect.value_namespace(), name.clone()))
            }
        };
        if !unique {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                "certified dependency native value symbols collide",
                spec.source.clone(),
            ));
        }
        if &dialect.dependency_value_spec(&spec.symbol) != spec {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                "dependency value metadata differs from its typed witness",
                spec.source.clone(),
            ));
        }
        let valid_type = match &spec.ty {
            TargetTypeRef::Primitive(_) => true,
            TargetTypeRef::Known(known) => catalogue.types.iter().any(|entry| {
                &entry.symbol == known
                    && entry.arity == 0
                    && entry.dependency.is_none()
                    && matches!(
                        entry.origin,
                        SymbolOrigin::Primitive
                            | SymbolOrigin::LanguagePrelude(_)
                            | SymbolOrigin::StandardLibrary(_)
                    )
            }),
            _ => false,
        };
        if !valid_type {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                "dependency values require context-free target types",
                spec.source.clone(),
            ));
        }
        if let Err(error) = dialect.verify_dependency_value_type(&spec.symbol, &spec.ty) {
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
    symbol: &D::DependencyValue,
) -> Option<ReferencePlan<D>> {
    catalogue
        .dependency_values
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
                namespace: dialect.value_namespace(),
                qualified_name,
                origin: SymbolOrigin::CertifiedDependency(spec.owner.clone()),
                policy,
                dependency: None,
            }
        })
}
