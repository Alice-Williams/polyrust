//! Typed binding allocation and deterministic collision handling.
use super::*;

#[derive(Clone)]
struct BindingCandidate<D: LinkerDialect> {
    symbol: BindableSymbolId<D>,
    requested: String,
    namespace: D::Namespace,
    scope: BindingScope,
    public: bool,
    source: SourceRef,
}

pub(super) fn allocate_bindings<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    catalogue: &SymbolCatalogue<D>,
    helpers: &BTreeSet<D::HelperId>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ResolvedBinding<D>> {
    let mut candidates: Vec<BindingCandidate<D>> = Vec::new();
    for (index, value) in package.generated_types().enumerate() {
        candidates.push(BindingCandidate {
            symbol: BindableSymbolId::Generated(GeneratedSymbolId::Type(
                GeneratedTypeId::from_index(index),
            )),
            requested: value.name.clone(),
            namespace: dialect.type_namespace(&value.kind),
            scope: BindingScope::Package,
            public: dialect.is_public(&value.visibility),
            source: value.source.clone(),
        });
    }
    for (index, value) in package.callables().enumerate() {
        candidates.push(BindingCandidate {
            symbol: BindableSymbolId::Generated(GeneratedSymbolId::Callable(
                GeneratedCallableId::from_index(index),
            )),
            requested: value.name.clone(),
            namespace: dialect.callable_namespace(),
            scope: BindingScope::Package,
            public: dialect.is_public(&value.visibility),
            source: value.source.clone(),
        });
    }
    for (index, value) in package.interface_methods().enumerate() {
        candidates.push(BindingCandidate {
            symbol: BindableSymbolId::Generated(GeneratedSymbolId::InterfaceMethod(
                GeneratedInterfaceMethodId::from_index(index),
            )),
            requested: value.name.clone(),
            namespace: dialect.member_namespace(),
            scope: BindingScope::Type(value.owner),
            public: true,
            source: value.source.clone(),
        });
    }
    for (index, value) in package.values().enumerate() {
        let id = GeneratedValueId::from_index(index);
        let scope = match binding_authority::value_scope(dialect, package, id) {
            Ok(scope) => scope,
            Err(error) => {
                diagnostics.push(Diagnostic::error(
                    error.code,
                    error.message,
                    value.source.clone(),
                ));
                continue;
            }
        };
        candidates.push(BindingCandidate {
            symbol: BindableSymbolId::Generated(GeneratedSymbolId::Value(id)),
            requested: value.name.clone(),
            namespace: dialect.value_namespace(),
            scope,
            public: dialect.is_public(&value.visibility),
            source: value.source.clone(),
        });
    }
    for helper in helpers {
        if let Some(spec) = catalogue.helper(helper) {
            candidates.push(BindingCandidate {
                symbol: BindableSymbolId::Helper(helper.clone()),
                requested: spec.alias_stem.clone(),
                namespace: spec.namespace.clone(),
                scope: BindingScope::Package,
                public: false,
                source: spec.source.clone(),
            });
        }
    }
    candidates.sort_by_key(|candidate| (!candidate.public, candidate.symbol.clone()));

    let mut occupied = BTreeSet::new();
    let mut bindings = Vec::new();
    for candidate in candidates {
        let base =
            match dialect.identifier_from_candidate(&candidate.requested, &candidate.namespace) {
                Ok(identifier) => identifier,
                Err(violation) => {
                    diagnostics.push(Diagnostic::error(
                        violation.code,
                        violation.message,
                        candidate.source,
                    ));
                    continue;
                }
            };
        let base_key = (
            candidate.scope,
            candidate.namespace.clone(),
            dialect.identifier_key(&base),
        );
        let identifier = if occupied.insert(base_key) {
            base
        } else if candidate.public {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                "public target name collides in its namespace",
                candidate.source.clone(),
            ));
            continue;
        } else {
            let mut suffix = 2u32;
            loop {
                let renamed = format!("{}_{}", candidate.requested, suffix);
                match dialect.identifier_from_candidate(&renamed, &candidate.namespace) {
                    Ok(identifier) => {
                        let key = (
                            candidate.scope,
                            candidate.namespace.clone(),
                            dialect.identifier_key(&identifier),
                        );
                        if occupied.insert(key) {
                            break identifier;
                        }
                    }
                    Err(violation) => {
                        diagnostics.push(Diagnostic::error(
                            violation.code,
                            violation.message,
                            candidate.source.clone(),
                        ));
                        break base;
                    }
                }
                suffix += 1;
            }
        };
        bindings.push(ResolvedBinding {
            symbol: candidate.symbol,
            identifier,
            namespace: candidate.namespace,
            scope: candidate.scope,
            public: candidate.public,
            source: candidate.source,
        });
    }
    bindings
}
