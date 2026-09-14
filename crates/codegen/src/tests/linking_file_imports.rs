// Exercise the actual linker and then mutate its otherwise-private witnesses.
use super::*;
use crate::linking::file_imports::derive;

fn linked(mode: CatalogueMode) -> LinkedTargetPackage<TestDialect> {
    TargetLinker::new(TestDialect(mode))
        .link_ast(&verified(file_graph_package(
            mode,
            SourceRole::Implementation,
            SourceRole::Implementation,
            Visibility::Public,
            false,
        )))
        .unwrap()
}

#[test]
fn one_directive_per_destination_without_changing_symbol_bindings() {
    let package = linked(CatalogueMode::GeneratedFiles);
    let file = &package.files[0];
    assert_eq!(file.dependencies.len(), 1);
    assert_eq!(file.file_imports.len(), 1);
    assert_eq!(file.file_imports[0].destination(), package.files[1].file());
    assert_eq!(
        file.file_imports[0].kind(),
        &ImportKind::File(package.files[1].path().clone())
    );
    assert!(file.imports.is_empty());
    assert!(package.files[1].file_imports.is_empty());
    assert_eq!(package.bindings, linked(CatalogueMode::Normal).bindings);
    assert_eq!(package, linked(CatalogueMode::GeneratedFiles));
}

#[test]
fn absent_policy_preserves_edges_without_emitting_directives() {
    let package = linked(CatalogueMode::Normal);
    assert_eq!(package.files[0].dependencies.len(), 1);
    assert!(
        package
            .files
            .iter()
            .all(|file| file.file_imports.is_empty())
    );
}

#[test]
fn coordinated_reference_edge_and_import_deletion_cannot_hide_a_dependency() {
    let mut package = linked(CatalogueMode::GeneratedFilesNoSpelling);
    let file = &mut package.files[0];
    assert!(!file.references.is_empty());
    file.references.clear();
    file.dependencies.clear();
    file.file_imports.clear();
    file.forward_declarations.clear();
    assert!(
        verify_linked_package(&package).is_err(),
        "reference-only dependencies must survive even when no spelling is consumed"
    );
}

#[test]
fn reference_inventory_authenticates_sources_multiplicity_and_helper_references() {
    let base = linked(CatalogueMode::GeneratedFilesNoSpelling);
    let mut changed_source = base.clone();
    changed_source.files[0].references[0].source = source("forged-provenance");
    let mut duplicate = base.clone();
    duplicate.files[0]
        .references
        .push(base.files[0].references[0].clone());
    for package in [changed_source, duplicate] {
        assert!(
            verify_linked_package(&package)
                .unwrap_err()
                .iter()
                .any(|error| error
                    .message
                    .contains("reference inventory is not exactly unresolved-root-derived"))
        );
    }

    let mode = CatalogueMode::Normal;
    let mut helpers = TargetLinker::new(TestDialect(mode))
        .link_ast(&verified(package(mode, full_symbols())))
        .unwrap();
    let runtime = helpers
        .files
        .iter_mut()
        .find(|file| !file.helpers.is_empty())
        .unwrap();
    assert!(!runtime.references.is_empty());
    runtime.references.pop();
    assert!(
        verify_linked_package(&helpers)
            .unwrap_err()
            .iter()
            .any(|error| error
                .message
                .contains("reference inventory is not exactly unresolved-root-derived"))
    );
}

fn rejects(package: &LinkedTargetPackage<TestDialect>) {
    let errors = verify_linked_package(package).unwrap_err();
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("file imports are not exactly reference-derived")),
        "{errors:?}"
    );
}

#[test]
fn missing_extra_duplicate_retargeted_and_kind_mutated_witnesses_fail() {
    let base = linked(CatalogueMode::GeneratedFiles);
    let mut missing = base.clone();
    missing.files[0].file_imports.clear();
    rejects(&missing);

    let mut extra = base.clone();
    extra.files[1].file_imports = base.files[0].file_imports.clone();
    rejects(&extra);

    let mut duplicate = base.clone();
    duplicate.files[0]
        .file_imports
        .push(base.files[0].file_imports[0].clone());
    rejects(&duplicate);

    let mut retargeted = base.clone();
    retargeted.files[0].file_imports[0].destination = base.files[0].file;
    rejects(&retargeted);

    let mut kind = base.clone();
    kind.files[0].file_imports[0].kind =
        ImportKind::File(RelativeOutputPath::new("forged.test").unwrap());
    rejects(&kind);

    // Removing both the witness and the advertised edge cannot conceal the
    // real generated-symbol reference used to reconstruct expected imports.
    missing.files[0].dependencies.clear();
    rejects(&missing);
}

#[test]
fn directive_policy_does_not_bypass_visibility_or_cycles() {
    for (mode, visibility, cycle) in [
        (CatalogueMode::RejectedFiles, Visibility::Public, false),
        (CatalogueMode::GeneratedFiles, Visibility::Private, false),
        (CatalogueMode::GeneratedFiles, Visibility::Public, true),
    ] {
        let package = verified(file_graph_package(
            mode,
            SourceRole::PublicApi,
            SourceRole::PublicApi,
            visibility,
            cycle,
        ));
        assert!(
            TargetLinker::new(TestDialect(mode))
                .link_ast(&package)
                .is_err()
        );
    }
}

#[test]
fn derivation_deduplicates_and_diagnoses_unknown_file_identity() {
    let package = linked(CatalogueMode::GeneratedFiles);
    let source = package.unresolved.file(package.files[0].file).unwrap();
    let destination = package.files[1].file;
    let mut errors = vec![];
    let imports = derive(
        &package.dialect,
        &package.unresolved,
        source,
        [destination, destination, destination],
        &mut errors,
    );
    assert!(errors.is_empty());
    assert_eq!(imports, package.files[0].file_imports);
    assert!(
        derive(
            &package.dialect,
            &package.unresolved,
            source,
            [TargetFileId::from_index(99)],
            &mut errors
        )
        .is_empty()
    );
    assert_eq!(errors.len(), 1);
}
