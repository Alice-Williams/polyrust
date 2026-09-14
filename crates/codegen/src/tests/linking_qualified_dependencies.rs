// Qualified dependency paths never allocate a consumer-local import binding.
use super::*;

fn assert_reference_checks(
    linked: &LinkedTargetPackage<TestDialect>,
    symbol: Callable,
    valid: bool,
) {
    let symbol = TargetSymbolRef::DependencyCallable(symbol);
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
                assert_eq!(
                    resolved_reference_matches(linked, reference, &imports),
                    valid
                );
            }
        }
    }
    assert!(found);
}

#[test]
fn same_member_names_keep_distinct_qualified_owners_without_imports() {
    let symbols = [
        Callable::QualifiedFirst,
        Callable::QualifiedOther,
        Callable::QualifiedOwned,
    ];
    let linked = link(&symbols).unwrap();
    let unused = link(&[]).unwrap();
    for _ in 0..3 {
        assert_eq!(linked, link(&symbols).unwrap());
    }
    assert!(verify_linked_package(&linked).is_ok());
    assert_eq!(linked.dependencies, unused.dependencies);
    assert_eq!(linked.bindings, unused.bindings);
    for (file, baseline) in linked.files.iter().zip(&unused.files) {
        assert_eq!(file.imports, baseline.imports);
    }
    for symbol in symbols {
        let DependencySpelling::Qualified(expected) = spec(&symbol).spelling else {
            panic!("qualified fixture");
        };
        let target = TargetSymbolRef::DependencyCallable(symbol.clone());
        let references = linked
            .files
            .iter()
            .flat_map(|file| &file.references)
            .filter(|reference| reference.symbol == target)
            .collect::<Vec<_>>();
        assert!(!references.is_empty());
        for reference in references {
            assert_eq!(
                reference.resolved,
                ResolvedReference::Qualified(expected.clone())
            );
        }
        assert_reference_checks(&linked, symbol, true);
    }
}

#[test]
fn fixed_and_qualified_symbols_may_share_a_member_name() {
    let linked = link(&[Callable::First, Callable::QualifiedFirst]).unwrap();
    assert!(verify_linked_package(&linked).is_ok());
    assert_reference_checks(&linked, Callable::First, true);
    assert_reference_checks(&linked, Callable::QualifiedFirst, true);
    let fixed = link(&[Callable::First]).unwrap();
    for (file, baseline) in linked.files.iter().zip(&fixed.files) {
        assert_eq!(file.imports, baseline.imports);
        assert!(file.imports.iter().all(|import| !import.symbols.contains(
            &TargetSymbolRef::DependencyCallable(Callable::QualifiedFirst)
        )));
    }
}

#[test]
fn duplicate_complete_qualified_paths_reject_even_when_unused() {
    for symbols in [vec![], vec![Callable::QualifiedConflict]] {
        let errors = link_mode(CatalogueMode::DependencyQualifiedConflict, &symbols).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("native symbols collide"))
        );
    }
}

#[test]
fn qualified_reference_substitution_fails_independent_verification() {
    let mut linked = link(&[Callable::QualifiedFirst]).unwrap();
    for file in &mut linked.files {
        for reference in &mut file.references {
            if reference.symbol == TargetSymbolRef::DependencyCallable(Callable::QualifiedFirst) {
                reference.resolved = ResolvedReference::Qualified(QualifiedName::DependencyOther);
            }
        }
    }
    assert_reference_checks(&linked, Callable::QualifiedFirst, false);
    assert!(verify_linked_package(&linked).is_err());
}

#[test]
fn coordinated_dependency_metadata_and_reference_changes_cannot_replace_witness() {
    let original = link(&[Callable::QualifiedFirst]).unwrap();
    for field in 0..3 {
        let mut linked = original.clone();
        let entry = linked
            .catalogue
            .dependency_callables
            .iter_mut()
            .find(|entry| entry.symbol == Callable::QualifiedFirst)
            .unwrap();
        match field {
            0 => entry.name = Identifier("replacement".into()),
            1 => entry.owner = Owner::Other,
            2 => {
                entry.spelling = DependencySpelling::Qualified(QualifiedName::DependencyEntry);
                for file in &mut linked.files {
                    for reference in &mut file.references {
                        if reference.symbol
                            == TargetSymbolRef::DependencyCallable(Callable::QualifiedFirst)
                        {
                            reference.resolved =
                                ResolvedReference::Qualified(QualifiedName::DependencyEntry);
                        }
                    }
                    for item in &mut file.items {
                        for reference in &mut item.references {
                            if *reference
                                == ResolvedReference::Qualified(QualifiedName::DependencyFirst)
                            {
                                *reference =
                                    ResolvedReference::Qualified(QualifiedName::DependencyEntry);
                            }
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        assert!(linked.catalogue.verify(&linked.dialect).is_err());
        assert!(verify_linked_package(&linked).is_err());
    }
}

#[test]
fn coordinated_qualified_to_fixed_policy_swap_cannot_replace_original_authority() {
    let mut linked = link(&[Callable::QualifiedFirst]).unwrap();
    let fixed = link(&[Callable::First]).unwrap();
    for (file, replacement) in linked.files.iter_mut().zip(fixed.files) {
        // Copy a consistent fixed-import allocation, then relabel only its witness.
        *file = replacement;
        for import in &mut file.imports {
            if import
                .symbols
                .remove(&TargetSymbolRef::DependencyCallable(Callable::First))
            {
                import.symbols.insert(TargetSymbolRef::DependencyCallable(
                    Callable::QualifiedFirst,
                ));
            }
        }
        for reference in &mut file.references {
            if reference.symbol == TargetSymbolRef::DependencyCallable(Callable::First) {
                reference.symbol = TargetSymbolRef::DependencyCallable(Callable::QualifiedFirst);
            }
        }
    }
    linked
        .catalogue
        .dependency_callables
        .retain(|entry| entry.symbol != Callable::First);
    let entry = linked
        .catalogue
        .dependency_callables
        .iter_mut()
        .find(|entry| entry.symbol == Callable::QualifiedFirst)
        .unwrap();
    entry.spelling = DependencySpelling::FixedImport(ImportKind::Value);
    // Internally consistent spelling is insufficient: the witness and AST disagree.
    assert_reference_checks(&linked, Callable::QualifiedFirst, true);
    assert!(linked.catalogue.verify(&linked.dialect).is_err());
    assert!(verify_linked_package(&linked).is_err());
}
