// Certified binding domains and compatible catalogue import coalescing.
use super::*;

#[test]
fn same_spelling_bindings_in_distinct_namespaces_never_coalesce() {
    for (mode, kind, owner) in [
        (
            CatalogueMode::DependencyValueSeparateNamespace,
            Kind::CallableCollision,
            dependency_tests::Owner::First,
        ),
        (
            CatalogueMode::DependencyValueSeparateOwnerNamespace,
            Kind::CallableOtherOwner,
            dependency_tests::Owner::Other,
        ),
    ] {
        let constant = symbol(kind);
        let callable = TargetSymbolRef::DependencyCallable(dependency_tests::Callable::First);
        let linked = link(
            mode,
            vec![constant.clone(), callable.clone(), constant.clone()],
        )
        .unwrap();
        assert!(verify_linked_package(&linked).is_ok());
        let mut checked = false;
        for file in &linked.files {
            let values = file
                .imports
                .iter()
                .filter(|item| item.symbols.contains(&constant))
                .collect::<Vec<_>>();
            let calls = file
                .imports
                .iter()
                .filter(|item| item.symbols.contains(&callable))
                .collect::<Vec<_>>();
            if values.is_empty() && calls.is_empty() {
                continue;
            }
            assert_eq!(values.len(), 1);
            assert_eq!(calls.len(), 1);
            assert_ne!(values[0].id, calls[0].id);
            assert_eq!(values[0].symbols, BTreeSet::from([constant.clone()]));
            assert_eq!(calls[0].symbols, BTreeSet::from([callable.clone()]));
            assert_eq!(values[0].binding, calls[0].binding);
            assert_eq!(
                values[0].origin,
                SymbolOrigin::CertifiedDependency(owner.clone())
            );
            checked = true;
        }
        assert!(checked);
    }
}

#[test]
fn coupled_domain_merge_and_empty_imports_fail_reconstruction() {
    for (mode, kind, callable) in [
        (
            CatalogueMode::DependencyValueSeparateNamespace,
            Kind::CallableCollision,
            TargetSymbolRef::DependencyCallable(dependency_tests::Callable::First),
        ),
        (
            CatalogueMode::Normal,
            Kind::First,
            TargetSymbolRef::KnownCallable(KnownCallable::Maximum),
        ),
    ] {
        let constant = symbol(kind);
        let original = link(mode, vec![constant.clone(), callable.clone()]).unwrap();
        let mut merged = original.clone();
        let mut changed = false;
        for file in &mut merged.files {
            let Some(value_index) = file
                .imports
                .iter()
                .position(|item| item.symbols.contains(&constant))
            else {
                continue;
            };
            let removed = file.imports.remove(value_index);
            let target = file
                .imports
                .iter_mut()
                .find(|item| item.symbols.contains(&callable))
                .unwrap();
            let target_id = target.id;
            target.symbols.extend(removed.symbols);
            for reference in &mut file.references {
                if let ResolvedReference::Imported { import, .. } = &mut reference.resolved
                    && *import == removed.id
                {
                    *import = target_id;
                }
            }
            for item in &mut file.items {
                for reference in &mut item.references {
                    if let ResolvedReference::Imported { import, .. } = reference
                        && *import == removed.id
                    {
                        *import = target_id;
                    }
                }
            }
            changed = true;
        }
        assert!(changed);
        assert!(
            verify_linked_package(&merged)
                .unwrap_err()
                .iter()
                .any(|error| error.message.contains("exactly one typed binding domain"))
        );
        let mut empty = original;
        let import = empty
            .files
            .iter_mut()
            .flat_map(|file| &mut file.imports)
            .find(|item| item.symbols.contains(&constant))
            .unwrap();
        import.symbols.clear();
        assert!(
            verify_linked_package(&empty)
                .unwrap_err()
                .iter()
                .any(|error| error.message.contains("exactly one typed binding domain"))
        );
    }
}

#[test]
fn known_type_and_constructor_preserve_shared_owner_import_across_namespaces() {
    let known = TargetSymbolRef::KnownType(KnownType::Clock);
    let constructor = TargetSymbolRef::KnownConstructor(KnownConstructor::NewClock);
    let linked = link(
        CatalogueMode::KnownTypeConstructorImport,
        vec![known.clone(), constructor.clone()],
    )
    .unwrap();
    assert!(verify_linked_package(&linked).is_ok());
    let mut checked = false;
    for file in &linked.files {
        let imports = file
            .imports
            .iter()
            .filter(|item| item.symbols.contains(&known) || item.symbols.contains(&constructor))
            .collect::<Vec<_>>();
        if imports.is_empty() {
            continue;
        }
        assert_eq!(imports.len(), 1);
        assert!(imports[0].symbols.contains(&known));
        assert!(imports[0].symbols.contains(&constructor));
        checked = true;
    }
    assert!(checked);
}
