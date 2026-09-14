// Package-derived authority must survive coordinated linked-data changes.
use super::*;

pub(super) fn derive(
    dialect: &TestDialect,
    package: &TargetAstPackage<TestDialect>,
) -> Result<SymbolCatalogue<TestDialect>, Vec<Diagnostic>> {
    if dialect.0 == CatalogueMode::EmptyPackageError {
        return Err(vec![]);
    }
    if dialect.0 == CatalogueMode::RejectedPackage {
        return Err(vec![link_error(
            DiagnosticCode::UnresolvedReference,
            "package catalogue has no dependency authority",
            "test-package",
        )]);
    }
    let mut result = dialect.symbol_catalogue();
    if dialect.0 == CatalogueMode::PackageDerived {
        let callable = result
            .callables
            .iter_mut()
            .find(|spec| spec.symbol == KnownCallable::Maximum)
            .unwrap();
        callable.name = Identifier(format!("maximum_for_{}_files", package.files().len()));
        callable.alias_stem = callable.name.0.clone();
        callable.source = source(&format!("package-with-{}-files", package.files().len()));
    }
    Ok(result)
}

fn link(mode: CatalogueMode) -> LinkedTargetPackage<TestDialect> {
    TargetLinker::new(TestDialect(mode))
        .link_ast(&verified(package(mode, full_symbols())))
        .unwrap()
}

fn rejects_catalogue(package: &LinkedTargetPackage<TestDialect>) {
    let errors = verify_linked_package(package).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("catalogue is not exactly")),
        "{errors:?}"
    );
}

#[test]
fn package_aware_catalogue_drives_linking_and_is_reconstructed_deterministically() {
    let mode = CatalogueMode::PackageDerived;
    let linked = link(mode);
    assert_eq!(linked, link(mode));
    assert!(verify_linked_package(&linked).is_ok());
    assert_ne!(linked.catalogue, linked.dialect.symbol_catalogue());
    let expected = Identifier(format!(
        "maximum_for_{}_files",
        linked.unresolved.files().len()
    ));
    assert!(
        linked
            .files
            .iter()
            .flat_map(|file| &file.imports)
            .any(|import| import.original_binding == expected)
    );
    let empty = TargetAstBuilder::new(TestDialect(mode)).build();
    assert_ne!(
        derive(&TestDialect(mode), &empty).unwrap(),
        linked.catalogue
    );
}

#[test]
fn static_catalogues_retain_their_original_default_result() {
    let linked = link(CatalogueMode::Normal);
    assert_eq!(linked.catalogue, linked.dialect.symbol_catalogue());
    assert!(verify_linked_package(&linked).is_ok());
}

#[test]
fn package_catalogue_failure_propagates_at_link_and_post_link_boundaries() {
    let mode = CatalogueMode::RejectedPackage;
    let errors = TargetLinker::new(TestDialect(mode))
        .link_ast(&verified(package(mode, full_symbols())))
        .unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("no dependency authority"))
    );
    let mut linked = link(CatalogueMode::Normal);
    linked.dialect = TestDialect(mode);
    linked.unresolved = package(mode, full_symbols());
    let errors = verify_linked_package(&linked).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("no dependency authority"))
    );
}

#[test]
fn unreferenced_missing_extra_and_changed_catalogue_metadata_rejects() {
    for mode in [CatalogueMode::Normal, CatalogueMode::PackageDerived] {
        let base = link(mode);
        let mut missing = base.clone();
        missing
            .catalogue
            .helpers
            .retain(|spec| spec.id != Helper::Unused);
        rejects_catalogue(&missing);
        let mut changed = base.clone();
        changed.catalogue.callables[0].source = source("invented-source");
        rejects_catalogue(&changed);
        let mut extra = base.clone();
        extra
            .catalogue
            .helpers
            .push(base.catalogue.helpers[0].clone());
        rejects_catalogue(&extra);
        let mut effect = base.clone();
        effect.catalogue.callables[0]
            .signature
            .effects
            .insert(TargetEffect::Nondeterminism);
        rejects_catalogue(&effect);
    }
}

#[test]
fn coordinated_catalogue_and_import_origin_changes_cannot_create_authority() {
    for mode in [CatalogueMode::Normal, CatalogueMode::PackageDerived] {
        let mut linked = link(mode);
        let symbol = KnownCallable::Maximum;
        let new_origin = SymbolOrigin::StandardLibrary(StandardLibrary::Time);
        let spec = linked
            .catalogue
            .callables
            .iter_mut()
            .find(|spec| spec.symbol == symbol)
            .unwrap();
        spec.origin = new_origin.clone();
        spec.dependency = None;
        let physical_name = spec.name.clone();
        for constructor in &mut linked.catalogue.constructors {
            if constructor.name == physical_name
                && constructor.policy == DependencyPolicy::Import(ImportKind::Value)
            {
                constructor.origin = new_origin.clone();
                constructor.dependency = None;
            }
        }
        for file in &mut linked.files {
            for import in &mut file.imports {
                if import
                    .symbols
                    .contains(&TargetSymbolRef::KnownCallable(symbol.clone()))
                {
                    import.origin = new_origin.clone();
                }
            }
        }
        // A coherent catalogue remains insufficient without the original input.
        assert!(linked.catalogue.verify(&linked.dialect).is_ok());
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
        rejects_catalogue(&linked);
    }
}

#[test]
fn an_empty_error_payload_never_becomes_success_during_post_link_verification() {
    let mode = CatalogueMode::EmptyPackageError;
    let errors = TargetLinker::new(TestDialect(mode))
        .link_ast(&verified(package(mode, full_symbols())))
        .unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("without a diagnostic"))
    );
    let mut linked = link(CatalogueMode::Normal);
    linked.dialect = TestDialect(mode);
    linked.unresolved = package(mode, full_symbols());
    let errors = verify_linked_package(&linked).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("without a diagnostic"))
    );
}

#[test]
fn changing_dialect_instances_cannot_bypass_package_authority() {
    for original in [
        CatalogueMode::Normal,
        CatalogueMode::PackageDerived,
        CatalogueMode::RejectedPackage,
        CatalogueMode::EmptyPackageError,
    ] {
        for replacement in [CatalogueMode::Normal, CatalogueMode::PackageDerived] {
            if original == replacement {
                continue;
            }
            let errors = TargetLinker::new(TestDialect(replacement))
                .link_ast(&verified(package(original, full_symbols())))
                .unwrap_err();
            assert!(
                errors.iter().any(|error| error
                    .message
                    .contains("linker dialect differs from original package dialect")),
                "{errors:?}"
            );
        }
    }
    for original in [CatalogueMode::Normal, CatalogueMode::PackageDerived] {
        for replacement in [
            CatalogueMode::Normal,
            CatalogueMode::PackageDerived,
            CatalogueMode::RejectedPackage,
            CatalogueMode::EmptyPackageError,
        ] {
            if original == replacement {
                continue;
            }
            let mut linked = link(original);
            linked.dialect = TestDialect(replacement);
            let errors = verify_linked_package(&linked).unwrap_err();
            assert!(
                errors.iter().any(|error| error
                    .message
                    .contains("linker dialect differs from original package dialect")),
                "{errors:?}"
            );
        }
    }
}
