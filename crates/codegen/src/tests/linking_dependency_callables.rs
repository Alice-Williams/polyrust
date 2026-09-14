// Fixed native references are independently checked after import resolution.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Callable {
    First,
    Second,
    OtherOwner,
    Collision,
    Missing,
    QualifiedFirst,
    QualifiedOther,
    QualifiedOwned,
    QualifiedConflict,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Owner {
    First,
    Other,
}

pub(super) fn spec(callable: &Callable) -> DependencyCallableSpec<TestDialect> {
    let (name, owner) = match callable {
        Callable::First => ("dependency", Owner::First),
        Callable::Second => ("dependency_second", Owner::First),
        Callable::OtherOwner => ("dependency", Owner::Other),
        Callable::Collision => ("entry", Owner::First),
        Callable::Missing => ("missing_dependency", Owner::First),
        Callable::QualifiedFirst => ("dependency", Owner::First),
        Callable::QualifiedOther | Callable::QualifiedConflict => ("dependency", Owner::Other),
        Callable::QualifiedOwned => ("entry", Owner::First),
    };
    let spelling = match callable {
        Callable::QualifiedFirst | Callable::QualifiedConflict => {
            DependencySpelling::Qualified(QualifiedName::DependencyFirst)
        }
        Callable::QualifiedOther => DependencySpelling::Qualified(QualifiedName::DependencyOther),
        Callable::QualifiedOwned => DependencySpelling::Qualified(QualifiedName::DependencyEntry),
        _ => DependencySpelling::FixedImport(ImportKind::Value),
    };
    DependencyCallableSpec {
        symbol: callable.clone(),
        owner,
        name: Identifier(name.into()),
        signature: signature(vec![i64_type()], i64_type()),
        spelling,
        source: source("certified-dependency"),
    }
}

pub(super) fn catalogue(mode: CatalogueMode) -> Vec<DependencyCallableSpec<TestDialect>> {
    let mut symbols = vec![
        Callable::First,
        Callable::Second,
        Callable::QualifiedFirst,
        Callable::QualifiedOther,
        Callable::QualifiedOwned,
    ];
    match mode {
        CatalogueMode::DependencyOwnedCollision => symbols.push(Callable::Collision),
        CatalogueMode::DependencyOwnerConflict => symbols.push(Callable::OtherOwner),
        CatalogueMode::DependencyQualifiedConflict => symbols.push(Callable::QualifiedConflict),
        _ => {}
    }
    symbols.iter().map(spec).collect()
}

fn link(symbols: &[Callable]) -> Result<LinkedTargetPackage<TestDialect>, Vec<Diagnostic>> {
    let mode = if symbols.contains(&Callable::Collision) {
        CatalogueMode::DependencyOwnedCollision
    } else if symbols.contains(&Callable::OtherOwner) {
        CatalogueMode::DependencyOwnerConflict
    } else if symbols.contains(&Callable::QualifiedConflict) {
        CatalogueMode::DependencyQualifiedConflict
    } else {
        CatalogueMode::Normal
    };
    link_mode(mode, symbols)
}

fn link_mode(
    mode: CatalogueMode,
    symbols: &[Callable],
) -> Result<LinkedTargetPackage<TestDialect>, Vec<Diagnostic>> {
    let ast = package(
        mode,
        symbols
            .iter()
            .cloned()
            .map(TargetSymbolRef::DependencyCallable)
            .collect(),
    );
    TargetLinker::new(TestDialect(mode)).link_ast(&verified(ast))
}

#[test]
fn unused_native_symbols_are_reserved_across_the_package() {
    for mode in [
        CatalogueMode::DependencyOwnedCollision,
        CatalogueMode::DependencyOwnerConflict,
    ] {
        assert!(link_mode(mode, &[]).is_err());
    }
}

#[test]
fn certified_symbols_link_with_exact_native_names_and_no_package_manager_metadata() {
    let linked = link(&[Callable::First, Callable::Second]).unwrap();
    assert_eq!(linked, link(&[Callable::First, Callable::Second]).unwrap());
    assert!(verify_linked_package(&linked).is_ok());
    for symbol in [Callable::First, Callable::Second] {
        let name = spec(&symbol).name;
        assert!(
            linked
                .files
                .iter()
                .flat_map(|file| &file.imports)
                .any(|import| import
                    .symbols
                    .contains(&TargetSymbolRef::DependencyCallable(symbol.clone()))
                    && import.binding == name
                    && import.original_binding == name
                    && import.origin == SymbolOrigin::CertifiedDependency(Owner::First))
        );
    }
    let normal = link(&[]).unwrap();
    assert_eq!(linked.dependencies, normal.dependencies);
}

#[test]
fn fixed_import_collisions_conflicting_owners_and_missing_symbols_reject() {
    for symbols in [
        vec![Callable::Collision],
        vec![Callable::First, Callable::OtherOwner],
        vec![Callable::Missing],
    ] {
        assert!(link(&symbols).is_err(), "{symbols:?}");
    }
}

#[test]
fn fixed_native_aliases_fail_the_independent_per_reference_verifier() {
    let mut linked = link(&[Callable::First]).unwrap();
    let symbol = TargetSymbolRef::DependencyCallable(Callable::First);
    for file in &mut linked.files {
        for import in &mut file.imports {
            if import.symbols.contains(&symbol) {
                import.binding = Identifier("invented_alias".into());
            }
        }
        for reference in &mut file.references {
            if reference.symbol == symbol
                && let ResolvedReference::Imported { binding, .. } = &mut reference.resolved
            {
                *binding = Identifier("invented_alias".into());
            }
        }
    }
    let mut found = false;
    for file in &linked.files {
        let imports = file
            .imports
            .iter()
            .map(|import| (import.id, import))
            .collect();
        for reference in &file.references {
            if reference.symbol == symbol {
                found = true;
                assert!(!resolved_reference_matches(&linked, reference, &imports));
            }
        }
    }
    assert!(found);
    assert!(verify_linked_package(&linked).is_err());
}

#[test]
fn exact_dependency_metadata_cannot_be_changed_or_removed_after_linking() {
    let linked = link(&[Callable::First]).unwrap();
    let mut changed = linked.clone();
    changed.catalogue.dependency_callables[0]
        .signature
        .parameters
        .clear();
    assert!(changed.catalogue.verify(&changed.dialect).is_err());
    assert!(verify_linked_package(&changed).is_err());
    let mut missing = linked.clone();
    missing.catalogue.dependency_callables.pop(); // Includes an unused symbol.
    assert!(verify_linked_package(&missing).is_err());
    let mut duplicate = linked;
    duplicate
        .catalogue
        .dependency_callables
        .push(spec(&Callable::First));
    assert!(duplicate.catalogue.verify(&duplicate.dialect).is_err());
    assert!(verify_linked_package(&duplicate).is_err());
}

#[test]
fn coordinated_owned_binding_and_reference_renames_do_not_replace_original_authority() {
    let mut linked = link(&[Callable::First]).unwrap();
    let binding = linked
        .bindings
        .iter_mut()
        .find(|binding| binding.identifier == Identifier("entry".into()))
        .expect("fixture contains the owned entry callable");
    let old = binding.identifier.clone();
    let new = Identifier("dependency".into());
    binding.identifier = new.clone();
    for file in &mut linked.files {
        for reference in &mut file.references {
            if reference.resolved == ResolvedReference::Local(old.clone()) {
                reference.resolved = ResolvedReference::Local(new.clone());
            }
        }
        for item in &mut file.items {
            for reference in &mut item.references {
                if *reference == ResolvedReference::Local(old.clone()) {
                    *reference = ResolvedReference::Local(new.clone());
                }
            }
        }
    }
    for file in &linked.files {
        let imports = file
            .imports
            .iter()
            .map(|import| (import.id, import))
            .collect();
        for reference in &file.references {
            assert!(resolved_reference_matches(&linked, reference, &imports));
        }
    }
    let diagnostics = verify_linked_package(&linked).unwrap_err();
    assert!(diagnostics.iter().any(|error| {
        error
            .message
            .contains("bindings are not exactly original-package-derived")
    }));
}

mod qualified {
    include!("linking_qualified_dependencies.rs");
}
