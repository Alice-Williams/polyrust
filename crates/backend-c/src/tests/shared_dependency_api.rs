//! Certificate-only dependency authority and exact public lookup regressions.
use super::*;
use crate::{
    ast::*,
    dialect::shared::{self, package_fixture, package_source_fixture::origins},
};
use portable_codegen::*;

fn fixture(scalar: CScalarType, layout: package_fixture::RecordLayout) -> package_fixture::Fixture {
    let (public, helper) = origins();
    package_fixture::with_source_shape(
        CGeneratedOrigin::RustSource(Arc::new(public)),
        CGeneratedOrigin::RustSource(Arc::new(helper)),
        layout,
        scalar,
    )
}

fn certify(fixture: &package_fixture::Fixture) -> RenderReadyPackage<CDialect> {
    certify_resolved_package(&CDialect, shared::package_projection_tests::linked(fixture)).unwrap()
}

#[test]
fn public_dependency_retains_exact_certificate_references_and_resource_evidence() {
    for scalar in [CScalarType::I32, CScalarType::Bool] {
        for layout in [
            package_fixture::RecordLayout::Absent,
            package_fixture::RecordLayout::Implementation,
        ] {
            let fixture = fixture(scalar, layout);
            let package = certify(&fixture);
            let expected = resources::measure_package(package.ast()).unwrap();
            let api = CDependencyApi::from_certificate(package.clone()).unwrap();
            assert_eq!(api.package(), &package);
            assert_eq!(api.root(), origins().0.crate_exports.root);
            assert_eq!(api.public_header().file(), fixture.public.file());
            assert_eq!(api.functions().count(), 1);
            let function = api.function(origins().0.declaration).unwrap();
            let defined = shared::c_defined_functions(&package)
                .find(|definition| definition.linkage() == CLinkage::External)
                .unwrap();
            assert_eq!(function.function(), defined.function());
            assert_eq!(function.symbol(), defined.name());
            assert_eq!(function.implementation(), defined.implementation());
            assert_eq!(function.signature(), fixture.public.signature());
            assert_eq!(function.public_header(), api.public_header());
            assert_eq!(function.stack_bound_bytes(), expected.total.frame_bound);
            assert!(function.stack_bound_bytes() > expected.total.function_frames[&fixture.public]);
            // A cloned function keeps the owning certificate alive after the API is gone.
            let retained = function.clone();
            drop(api);
            assert_eq!(retained.symbol(), defined.name());
            assert_eq!(retained.stack_bound_bytes(), expected.total.frame_bound);
        }
    }
}

#[test]
fn public_lookup_does_not_expose_private_missing_or_foreign_identities() {
    let fixture = fixture(CScalarType::I32, package_fixture::RecordLayout::Absent);
    let api = CDependencyApi::from_certificate(certify(&fixture)).unwrap();
    let (public, helper) = origins();
    for id in [
        helper.declaration,
        public.module,
        RustDeclarationId {
            definition_path_hash: 999,
            ..public.declaration
        },
        RustDeclarationId {
            crate_id: 8,
            ..public.declaration
        },
    ] {
        assert!(api.function(id).is_none(), "{id:?}");
    }
    let function = api.function(public.declaration).unwrap();
    let consumer = CRegistry::new();
    assert!(
        CExpressions::new(&consumer)
            .direct(function.function().clone())
            .is_err()
    );
}

#[test]
fn altered_cached_stack_evidence_cannot_certify_a_consumer() {
    let input = fixture(CScalarType::I32, package_fixture::RecordLayout::Absent);
    let api = CDependencyApi::from_certificate(certify(&input)).unwrap();
    let original = api.functions().next().unwrap();
    let actual = original.stack_bound_bytes();
    for forged in [0, actual - 1, actual + 1, u64::MAX] {
        // Deliberately violate private module invariants to exercise the
        // independent reconstruction; no public API can author this summary.
        let mut altered = original.clone();
        let mut authority = altered.authority.as_ref().clone();
        authority.stack_bound_bytes = forged;
        altered.authority = Arc::new(authority);
        for call in [None, Some(0)] {
            let consumer = shared::dependency_fixture::fixture(
                90,
                &[CScalarType::I32],
                &[altered.clone()],
                &[call],
            );
            let linked = shared::dependency_fixture::linked(&consumer);
            if forged == u64::MAX && call.is_some() {
                let error = resources::measure_package(&linked).unwrap_err();
                assert!(error.contains("overflow"));
            }
            let errors = match certify_resolved_package(&CDialect, linked) {
                Err(errors) => errors,
                Ok(_) => panic!("altered cached dependency bound was accepted"),
            };
            assert!(
                errors
                    .iter()
                    .any(|error| error.message.contains("stack evidence differs"))
            );
        }
    }
}

#[test]
fn independent_certificates_do_not_substitute_function_registry_authority() {
    let first = fixture(CScalarType::I32, package_fixture::RecordLayout::Absent);
    let second = fixture(CScalarType::I32, package_fixture::RecordLayout::Absent);
    let first = CDependencyApi::from_certificate(certify(&first)).unwrap();
    let second = CDependencyApi::from_certificate(certify(&second)).unwrap();
    let id = origins().0.declaration;
    assert_eq!(first.function(id).unwrap().declaration(), id);
    assert_ne!(
        first.function(id).unwrap().function(),
        second.function(id).unwrap().function()
    );
    assert!(!Arc::ptr_eq(&first.authority, &second.authority));
}

#[test]
fn certificate_alone_does_not_admit_non_package_or_non_rust_scalar_apis() {
    let (registry, source) = shared::tests::fixture(CScalarType::I32);
    let unresolved = shared::project_c_package(registry, vec![source]).unwrap();
    let checked = verify_unresolved_package(&CDialect, unresolved).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let single = certify_resolved_package(&CDialect, linked).unwrap();
    assert!(
        CDependencyApi::from_certificate(single)
            .unwrap_err()
            .contains("header/source pair")
    );
    assert!(
        CDependencyApi::from_certificate(certify(&package_fixture::fixture()))
            .unwrap_err()
            .contains("Rust-source")
    );
    let wide_api = fixture(CScalarType::Int, package_fixture::RecordLayout::Absent);
    assert!(
        CDependencyApi::from_certificate(certify(&wide_api))
            .unwrap_err()
            .contains("admitted-scalar-parameter/scalar-or-void-result proof")
    );
}

#[test]
fn scalar_dependency_header_layouts_reject_before_certification() {
    let fixture = fixture(CScalarType::I32, package_fixture::RecordLayout::Header);
    let error = shared::project_c_package(fixture.registry, fixture.files).unwrap_err();
    assert!(error.iter().any(|error| {
        error
            .message
            .contains("exact registered scalar-result layout")
    }));
}

#[derive(Clone, Copy, Debug)]
enum Mutation {
    MissingBinding,
    ExtraBinding,
    PrivateBinding,
    WrongNamespace,
    ForeignDeclaration,
    ForeignModule,
    RestrictedPublic,
    WrongReachability,
}

#[test]
fn coordinated_metadata_changes_cannot_manufacture_dependency_apis() {
    for mutation in [
        Mutation::MissingBinding,
        Mutation::ExtraBinding,
        Mutation::PrivateBinding,
        Mutation::WrongNamespace,
        Mutation::ForeignDeclaration,
        Mutation::ForeignModule,
        Mutation::RestrictedPublic,
        Mutation::WrongReachability,
    ] {
        let (mut public, mut helper) = origins();
        let root = public.crate_exports.root;
        let graph = Arc::make_mut(&mut public.crate_exports);
        let entries = graph.modules.get_mut(&root).unwrap();
        match mutation {
            Mutation::MissingBinding => entries.clear(),
            Mutation::ExtraBinding | Mutation::PrivateBinding | Mutation::ForeignDeclaration => {
                let target = match mutation {
                    Mutation::PrivateBinding => helper.declaration,
                    Mutation::ForeignDeclaration => RustDeclarationId {
                        crate_id: 8,
                        ..public.declaration
                    },
                    _ => RustDeclarationId {
                        definition_path_hash: 999,
                        ..public.declaration
                    },
                };
                entries.insert(
                    RustExportName {
                        namespace: RustExportNamespace::Value,
                        name: "extra".into(),
                    },
                    RustExportTarget::Declaration(target),
                );
            }
            Mutation::WrongNamespace => {
                let (_, target) = entries.pop_first().unwrap();
                entries.insert(
                    RustExportName {
                        namespace: RustExportNamespace::Macro,
                        name: "wrong".into(),
                    },
                    target,
                );
            }
            Mutation::ForeignModule => {
                entries.insert(
                    RustExportName {
                        namespace: RustExportNamespace::Type,
                        name: "foreign".into(),
                    },
                    RustExportTarget::Module(RustDeclarationId {
                        crate_id: 8,
                        ..root
                    }),
                );
            }
            Mutation::RestrictedPublic => public.visibility = RustVisibility::RestrictedTo(root),
            Mutation::WrongReachability => public.externally_reachable = false,
        }
        helper.crate_exports = public.crate_exports.clone();
        let fixture = package_fixture::with_origins(
            CGeneratedOrigin::RustSource(Arc::new(public)),
            CGeneratedOrigin::RustSource(Arc::new(helper)),
        );
        // Valid C syntax/certification does not authenticate arbitrary source API claims.
        let certified = certify(&fixture);
        assert!(
            CDependencyApi::from_certificate(certified).is_err(),
            "{mutation:?}"
        );
    }
}
