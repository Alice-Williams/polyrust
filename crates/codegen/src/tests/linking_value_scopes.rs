// Owner scope is original-package authority, not editable linked metadata.
use super::*;

fn link(mode: CatalogueMode) -> Result<LinkedTargetPackage<TestDialect>, Vec<Diagnostic>> {
    TargetLinker::new(TestDialect(mode)).link_ast(&verified(package(mode, full_symbols())))
}

#[test]
fn aggregate_owners_separate_values_without_renaming() {
    let linked = link(CatalogueMode::ValueOwners(true)).unwrap();
    for (index, binding) in linked
        .bindings
        .iter()
        .filter(|b| {
            matches!(
                b.symbol,
                BindableSymbolId::Generated(GeneratedSymbolId::Value(_))
            )
        })
        .enumerate()
    {
        assert_eq!(binding.identifier, Identifier("temporary".into()));
        assert_eq!(
            binding.scope,
            BindingScope::Type(GeneratedTypeId::from_index(index))
        );
    }
    verify_linked_package(&linked).unwrap();
    assert_eq!(linked, link(CatalogueMode::ValueOwners(true)).unwrap());
}

#[test]
fn absent_type_owners_cannot_allocate_bindings() {
    let errors = link(CatalogueMode::ValueOwners(false)).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("absent owning type")),
        "{errors:?}"
    );
}

#[test]
fn post_link_verification_reconstructs_ownership_even_without_name_collisions() {
    for scope in [
        BindingScope::Package,
        BindingScope::Type(GeneratedTypeId::from_index(0)),
    ] {
        let mut linked = link(CatalogueMode::ValueOwners(true)).unwrap();
        let binding = linked
            .bindings
            .iter_mut()
            .find(|b| {
                b.symbol
                    == BindableSymbolId::Generated(GeneratedSymbolId::Value(
                        GeneratedValueId::from_index(1),
                    ))
            })
            .unwrap();
        binding.scope = scope;
        let errors = verify_linked_package(&linked).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("not exactly original-package-derived")),
            "{errors:?}"
        );
    }
}
