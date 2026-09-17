use super::*;

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
fn unnamed_libraries_are_deduplicated_without_symbol_bindings() {
    let package = linked(CatalogueMode::RequiredLibraries(false));
    let baseline = linked(CatalogueMode::Normal);
    assert_eq!(package.bindings, baseline.bindings);
    for file in &package.files {
        assert!(file.imports.is_empty());
        assert_eq!(file.library_imports.len(), 1);
        assert_eq!(file.library_imports[0].library(), &StandardLibrary::Time);
        assert_eq!(file.library_imports[0].kind(), &ImportKind::Value);
    }
    assert_eq!(package, linked(CatalogueMode::RequiredLibraries(false)));
}
#[test]
fn unnamed_library_imports_reject_deletion_forgery_and_duplication() {
    let baseline = linked(CatalogueMode::RequiredLibraries(false));
    let mut removed = baseline.clone();
    removed.files[0].library_imports.clear();
    let mut duplicate = baseline.clone();
    duplicate.files[0]
        .library_imports
        .push(baseline.files[0].library_imports[0].clone());
    let mut origin = baseline.clone();
    origin.files[0].library_imports[0].library = StandardLibrary::Runtime;
    let mut kind = baseline.clone();
    kind.files[0].library_imports[0].kind = ImportKind::Type;
    let mut inserted = linked(CatalogueMode::Normal);
    inserted.files[0].library_imports = baseline.files[0].library_imports.clone();
    for changed in [removed, duplicate, origin, kind, inserted] {
        assert!(
            verify_linked_package(&changed)
                .unwrap_err()
                .iter()
                .any(|diagnostic| diagnostic
                    .message
                    .contains("library imports are not exactly metadata-derived"))
        );
    }
}
#[test]
fn unsupported_unnamed_library_is_a_link_error() {
    let mode = CatalogueMode::RequiredLibraries(true);
    let package = verified(file_graph_package(
        mode,
        SourceRole::Implementation,
        SourceRole::Implementation,
        Visibility::Public,
        false,
    ));
    assert!(
        TargetLinker::new(TestDialect(mode))
            .link_ast(&package)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unsupported library"))
    );
}
