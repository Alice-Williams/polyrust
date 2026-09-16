//! Explicit source identity is retained but never substitutes for certification.
use super::{
    CDependencyApi, CDialect, CStructuralRenderer,
    constant_producer_tests::certify,
    documentation,
    owned_constant_fixture::{self, Shape},
    project_c_package,
};
use crate::ast::*;
use portable_codegen::*;
use std::sync::Arc;

#[path = "shared_source_package_graph.rs"]
mod graph;
#[path = "shared_source_package_proof.rs"]
mod proof;

fn explicit(shape: Shape) -> owned_constant_fixture::Fixture {
    owned_constant_fixture::with_registration(shape, |registry, header, exports| {
        registry.register_source_package(header, exports).unwrap();
    })
}

fn file(registry: &mut CRegistry, path: &str, role: CFileRole) -> CFileRef {
    registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(path).unwrap(),
            role,
        })
        .unwrap()
}

#[test]
fn registration_is_branded_single_assignment_and_survives_freeze() {
    let mut registry = CRegistry::new();
    let header = file(
        &mut registry,
        "polyrust_package.h",
        CFileRole::GeneratedPublicHeader,
    );
    let source = file(&mut registry, "package.c", CFileRole::GeneratedSource);
    let graph = super::package_source_fixture::origins().0.crate_exports;
    assert_eq!(
        registry.register_source_package(&source, graph.clone()),
        Err(CRegistryError::WrongOwner)
    );
    let mut foreign = CRegistry::new();
    let other = file(
        &mut foreign,
        "polyrust_package.h",
        CFileRole::GeneratedPublicHeader,
    );
    assert_eq!(
        registry.register_source_package(&other, graph.clone()),
        Err(CRegistryError::CrossRegistry)
    );
    assert!(registry.source_package().is_none());
    registry
        .register_source_package(&header, graph.clone())
        .unwrap();
    assert_eq!(
        registry.register_source_package(&header, graph.clone()),
        Err(CRegistryError::DuplicateRegistration)
    );
    let frozen = registry.freeze();
    let package = frozen.registrations().source_package().unwrap();
    assert_eq!(package.header(), &header);
    assert!(Arc::ptr_eq(package.exports(), &graph));
}

#[test]
fn explicit_constant_and_mixed_packages_render_exactly_like_inferred_packages() {
    let render = |fixture: &owned_constant_fixture::Fixture| {
        let checked = certify(fixture);
        let api = CDependencyApi::from_certificate(checked.clone()).unwrap();
        assert_eq!(api.constants().count(), fixture.objects.len());
        assert_eq!(api.functions().count(), fixture.functions.len());
        render_certified_package(&CStructuralRenderer, &checked)
            .unwrap()
            .files()
            .iter()
            .map(|file| (file.path().to_owned(), file.contents().clone()))
            .collect::<Vec<_>>()
    };
    for shape in [Shape::ConstantsOnly, Shape::Mixed] {
        let explicit = explicit(shape);
        let source = explicit.registry.registrations().source_package().unwrap();
        let api = CDependencyApi::from_certificate(certify(&explicit)).unwrap();
        assert_eq!(api.root(), source.exports().root);
        assert_eq!(api.public_header().file(), source.header());
        assert_eq!(
            render(&explicit),
            render(&owned_constant_fixture::fixture(shape))
        );
        let mut reversed = explicit;
        reversed.files.reverse();
        assert_eq!(
            render(&reversed),
            render(&owned_constant_fixture::fixture(shape))
        );
    }
}

#[test]
fn explicit_graph_disagreement_never_falls_back_to_a_declaration() {
    let fixture =
        owned_constant_fixture::with_registration(Shape::Mixed, |registry, header, mut exports| {
            Arc::make_mut(&mut exports).root.crate_id += 1;
            registry.register_source_package(header, exports).unwrap();
        });
    let errors = project_c_package(fixture.registry, fixture.files).unwrap_err();
    assert!(format!("{errors:?}").contains("disagrees with explicit source-package provenance"));
}

#[test]
fn same_root_with_different_bindings_is_not_equal_provenance() {
    let fixture = owned_constant_fixture::with_registration(
        Shape::ConstantsOnly,
        |registry, header, mut exports| {
            let graph = Arc::make_mut(&mut exports);
            graph.modules.get_mut(&graph.root).unwrap().clear();
            registry.register_source_package(header, exports).unwrap();
        },
    );
    let errors = project_c_package(fixture.registry, fixture.files).unwrap_err();
    assert!(format!("{errors:?}").contains("disagrees with explicit source-package provenance"));
}

#[test]
fn explicit_metadata_cannot_retarget_or_omit_its_output_header() {
    let fixture = explicit(Shape::ConstantsOnly);
    let registry = fixture.registry.registrations();
    assert!(super::source_package::check(registry, &fixture.files[1..]).is_err());
    assert!(
        super::source_package::check(
            registry,
            &[fixture.files[1].clone(), fixture.files[1].clone()]
        )
        .is_err()
    );
    let other = explicit(Shape::ConstantsOnly);
    assert!(super::source_package::check(registry, &other.files).is_err());
    assert!(
        super::source_package::check(
            registry,
            &[fixture.files[0].clone(), other.files[1].clone()]
        )
        .is_err()
    );
}

/// No invented owner: this graph and pair contain zero declarations.
fn empty_package(exports: Arc<RustCrateExports>) -> (CFrozenRegistry, Vec<CSourceFile>) {
    let mut registry = CRegistry::new();
    let header = file(
        &mut registry,
        "polyrust_package.h",
        CFileRole::GeneratedPublicHeader,
    );
    let source = file(&mut registry, "package.c", CFileRole::GeneratedSource);
    registry.register_source_package(&header, exports).unwrap();
    let files = [header, source]
        .into_iter()
        .map(|file| {
            CDeclarations::new(&registry, file)
                .unwrap()
                .source_file(vec![])
                .unwrap()
        })
        .collect();
    (registry.freeze(), files)
}

#[test]
fn docs_without_owned_declarations_do_not_enable_empty_package_publication() {
    let mut exports = super::package_source_fixture::origins().0.crate_exports;
    let graph = Arc::make_mut(&mut exports);
    let bindings = graph.modules.get_mut(&graph.root).unwrap();
    bindings.clear();
    bindings.insert(
        RustExportName {
            namespace: RustExportNamespace::Value,
            name: "alias".into(),
        },
        RustExportTarget::Declaration(RustDeclarationId {
            crate_id: 8,
            definition_path_hash: 10,
        }),
    );
    let (registry, files) = empty_package(exports.clone());
    assert!(registry.registrations().inventory().is_empty());
    let docs = documentation::lower_registered_package(registry.registrations(), &files).unwrap();
    assert_eq!(
        docs[files[0].identity()]
            .modules()
            .map(CComment::text)
            .collect::<Vec<_>>(),
        ["public root"]
    );
    assert_eq!(docs[files[1].identity()].all().count(), 0);
    assert!(project_c_package(registry, files).is_err());
    let graph = Arc::make_mut(&mut exports);
    graph.modules.get_mut(&graph.root).unwrap().clear();
    let (registry, files) = empty_package(exports);
    assert!(documentation::lower_registered_package(registry.registrations(), &files).is_ok());
    assert!(project_c_package(registry, files).is_err());
}
