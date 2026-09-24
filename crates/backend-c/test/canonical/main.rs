//! Checked, type-only C publication; no compiler-source admission is asserted.
mod assertions;
mod fixture;
mod native;
use fixture::*;
use portable_backend_c::{ast::*, dialect::*};
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

fn exports(root: RustDeclarationId) -> Arc<RustCrateExports> {
    Arc::new(RustCrateExports {
        root,
        modules: BTreeMap::new(),
        module_ancestries: BTreeMap::new(),
    })
}

#[test]
fn exact_owner_survives_certification_and_has_only_structural_imports() {
    for maximum in [false, true] {
        let mut value = fixture(maximum, Fault::None);
        register(&mut value, Fault::None).unwrap();
        let descriptor = value.registry.canonical_type_package().unwrap().clone();
        assert!(value.registry.source_package().is_none());
        assert_eq!(descriptor.facts(), facts(maximum));
        assert_eq!(
            descriptor.owner(),
            TargetPackageOwner::CanonicalInstance {
                instance: facts(maximum).instance().key(),
                profile: PROFILE,
            }
        );
        let certificate = certify(value).unwrap();
        assert_eq!(c_canonical_type_package(&certificate), Some(&descriptor));
        assert!(c_source_package(&certificate).is_none());
        let header = certificate
            .ast()
            .files()
            .iter()
            .find(|f| f.module() == descriptor.header())
            .unwrap();
        let source = certificate
            .ast()
            .files()
            .iter()
            .find(|f| f.module() == descriptor.implementation())
            .unwrap();
        assert!(header.file_imports().is_empty());
        assert_eq!(source.file_imports().len(), 1);
        assert_eq!(source.file_imports()[0].destination(), header.file());
        let output = render_certified_package(&CStructuralRenderer, &certificate).unwrap();
        assert_eq!(output.files().len(), 2);
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text");
            };
            assert!(!text.contains("runtime") && !text.contains("malloc"));
            if file.path().ends_with(".h") {
                assert_eq!(text.matches("_Static_assert").count(), 10);
                assert_eq!(text.matches("struct ").count(), 1);
                let guards: Vec<_> = text
                    .lines()
                    .filter(|line| line.starts_with("#ifndef "))
                    .collect();
                assert_eq!(guards.len(), 1);
                assert!(guards[0].len() < 256);
            } else {
                assert_eq!(text.matches("#include \"").count(), 1);
            }
        }
        let api = CDependencyApi::from_certificate(certificate).unwrap();
        assert_eq!(api.owner(), descriptor.owner());
        assert_eq!(api.source_root(), None);
    }
}

#[test]
fn malformed_type_owners_never_obtain_a_certificate() {
    for fault in [
        Fault::HeaderName,
        Fault::SourceName,
        Fault::RecordName,
        Fault::TagName,
        Fault::ValueName,
        Fault::TagType,
        Fault::ValueType,
        Fault::ReversedMembers,
        Fault::ExtraMember,
        Fault::ExtraFile,
        Fault::ExtraRecord,
        Fault::MissingDeclaration,
        Fault::DuplicateDeclaration,
        Fault::HeaderComment,
        Fault::SourceComment,
        Fault::DifferentFacts,
        Fault::NoProfile,
        Fault::ForgedCoreOrigin,
    ] {
        let mut value = fixture(false, fault);
        if register(&mut value, fault).is_ok() {
            assert!(certify(value).is_err(), "{fault:?} escaped certification");
        } else {
            assert!(
                matches!(fault, Fault::ExtraMember),
                "unexpected early rejection: {fault:?}"
            );
        }
    }
}

#[test]
fn source_and_type_ownership_are_mutually_exclusive_in_both_orders() {
    let mut value = fixture(false, Fault::None);
    register(&mut value, Fault::None).unwrap();
    assert_eq!(
        register(&mut value, Fault::None),
        Err(CRegistryError::DuplicateRegistration)
    );
    assert_eq!(
        value
            .registry
            .register_source_package(&value.header, exports(value.facts.instance().core_root())),
        Err(CRegistryError::DuplicateRegistration)
    );
    let mut value = fixture(false, Fault::None);
    value
        .registry
        .register_source_package(&value.header, exports(value.facts.instance().core_root()))
        .unwrap();
    assert_eq!(
        register(&mut value, Fault::None),
        Err(CRegistryError::DuplicateRegistration)
    );
    assert!(certify(value).is_err()); // Source ownership cannot open the type-only profile.
}

#[test]
fn independent_registries_and_input_file_order_cannot_change_output() {
    let mut left = fixture(false, Fault::None);
    register(&mut left, Fault::None).unwrap();
    let left = render_certified_package(&CStructuralRenderer, &certify(left).unwrap()).unwrap();
    let mut right = fixture(false, Fault::None);
    right.files.reverse();
    register(&mut right, Fault::None).unwrap();
    let right = render_certified_package(&CStructuralRenderer, &certify(right).unwrap()).unwrap();
    assert_eq!(left, right);
}

#[test]
fn role_references_cannot_cross_registries_or_swap_file_roles() {
    let mut left = fixture(false, Fault::None);
    let right = fixture(false, Fault::None);
    assert_eq!(
        left.registry.register_canonical_type_package(
            &left.header,
            &right.implementation,
            PROFILE,
            left.facts,
            &left.record
        ),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        left.registry.register_canonical_type_package(
            &left.implementation,
            &left.header,
            PROFILE,
            left.facts,
            &left.record
        ),
        Err(CRegistryError::WrongOwner)
    );
    assert!(left.registry.canonical_type_package().is_none());
    register(&mut left, Fault::None).unwrap();
    certify(left).unwrap();
}
