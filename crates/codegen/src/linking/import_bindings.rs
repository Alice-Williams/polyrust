//! Certified bindings retain namespaces; known owner imports preserve coalescing.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Domain<D: LinkerDialect> {
    Catalogue,
    Certified(D::Namespace),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Key<D: LinkerDialect> {
    pub domain: Domain<D>,
    pub kind: D::ImportKind,
    pub name: D::Identifier,
}

pub(super) fn domain<D: LinkerDialect>(
    symbol: &TargetSymbolRef<D>,
    namespace: D::Namespace,
) -> Domain<D> {
    match symbol {
        TargetSymbolRef::DependencyCallable(_) | TargetSymbolRef::DependencyValue(_) => {
            Domain::Certified(namespace)
        }
        _ => Domain::Catalogue,
    }
}

pub(super) fn reconstruct<D: LinkerDialect>(
    dialect: &D,
    catalogue: &SymbolCatalogue<D>,
    import: &ResolvedImport<D>,
) -> Option<Domain<D>> {
    let mut domains = BTreeSet::new();
    for symbol in &import.symbols {
        domains.insert(domain(
            symbol,
            reference_plan(dialect, catalogue, symbol)?.namespace,
        ));
    }
    if domains.len() == 1 {
        domains.into_iter().next()
    } else {
        None
    }
}
