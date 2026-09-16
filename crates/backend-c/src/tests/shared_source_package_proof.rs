//! Source metadata cannot be dropped from independent projection reconstruction.
use super::*;

#[test]
fn every_owned_registration_is_checked_not_only_public_definitions() {
    let fixture = owned_constant_fixture::with_registration(
        Shape::ConstantsOnly,
        |registry, header, exports| {
            registry.register_source_package(header, exports).unwrap();
            let mut origin = super::super::package_source_fixture::origins().0;
            origin.declaration.crate_id = 999;
            registry
                .register_function(
                    header,
                    CDeclarationKey {
                        name: CIdentifier::new("unused_foreign_origin").unwrap(),
                        origin: CGeneratedOrigin::RustSource(Arc::new(origin)),
                    },
                    CFunctionType::new(CReturnType::Void, vec![]),
                )
                .unwrap();
        },
    );
    let errors = project_c_package(fixture.registry, fixture.files).unwrap_err();
    assert!(format!("{errors:?}").contains("disagrees with explicit source-package provenance"));
}

#[test]
fn shared_verifier_reconstructs_documentation_from_explicit_package_metadata() {
    let fixture = explicit(Shape::ConstantsOnly);
    let package = project_c_package(fixture.registry, fixture.files).unwrap();
    verify_unresolved_package(&CDialect, package.clone()).unwrap();
    let rebuild = |strip_docs| {
        let mut builder = TargetAstBuilder::new(CDialect);
        for ty in package.generated_types() {
            builder.generated_type(ty.clone());
        }
        for callable in package.callables() {
            builder.callable(callable.clone());
        }
        for value in package.values() {
            builder.value(value.clone());
        }
        for file in package.files() {
            let mut units = file.items().to_vec();
            if strip_docs && file.module().key().role == CFileRole::GeneratedPublicHeader {
                Arc::make_mut(&mut units[0].data).documentation = Default::default();
            }
            builder.file(TargetFile::new(
                file.path().clone(),
                file.role(),
                file.module().clone(),
                *file.placement(),
                units,
                file.source_kind().clone(),
                file.source().clone(),
            ));
        }
        for group in package.groups() {
            builder.group(group.clone());
        }
        builder.build()
    };
    verify_unresolved_package(&CDialect, rebuild(false)).unwrap();
    assert!(verify_unresolved_package(&CDialect, rebuild(true)).is_err());
}
