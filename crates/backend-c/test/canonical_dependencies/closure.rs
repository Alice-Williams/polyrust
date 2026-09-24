//! Unused imports and relays must preserve complete original owner authority.
use super::{
    fixture::api,
    source::{root, source},
};
use portable_backend_c::{ast::*, dialect::*};
use portable_codegen::*;

#[test]
fn same_core_types_do_not_claim_core_source_ownership_or_zero_executable_cost() {
    let left = api(0);
    let right = api(100);
    let proofs: Vec<_> = left.structs().chain(right.structs()).cloned().collect();
    // The source shares core's crate ID, but neither canonical type owns that crate.
    let consumer = source(root(7), &proofs, &[], false).unwrap();
    assert_eq!(consumer.source_root(), Some(root(7)));
    assert_eq!(consumer.dependencies().unwrap().len(), 2);
    assert!(
        consumer.structs().next().is_none(),
        "unused imports are not public exports"
    );
    assert!(consumer.functions().next().unwrap().stack_bound_bytes() > 0);
    assert!(left.dependencies().unwrap().is_empty());
    render_certified_package(&CStructuralRenderer, consumer.package()).unwrap();
}

#[test]
fn record_signature_relays_keep_original_type_and_member_authority() {
    let original = api(0);
    let proof = original.structs().next().unwrap().clone();
    let first = source(root(10), std::slice::from_ref(&proof), &[], true).unwrap();
    let second = source(
        root(11),
        std::slice::from_ref(&proof),
        &[first.functions().next().unwrap().clone()],
        true,
    )
    .unwrap();
    for relay in [&first, &second] {
        assert_eq!(relay.structure(proof.record()), Some(&proof));
        assert_eq!(relay.structs().next().unwrap().members(), proof.members());
        let output = render_certified_package(&CStructuralRenderer, relay.package()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(!text.contains(&format!("struct {} {{", proof.symbol().as_str())));
        }
    }
    assert_eq!(second.dependencies().unwrap().len(), 2);
    assert!(
        second.functions().next().unwrap().stack_bound_bytes()
            > first.functions().next().unwrap().stack_bound_bytes()
    );
}

#[test]
fn unused_type_owners_survive_scalar_relays_and_reconcile_diamonds() {
    let original = api(0);
    let replacement = CDependencyApi::from_certificate(original.package().clone()).unwrap();
    let other_instance = api(100);
    let left = source(
        root(20),
        &[original.structs().next().unwrap().clone()],
        &[],
        false,
    )
    .unwrap();
    for (right_owner, valid) in [
        (&original, true),
        (&replacement, false),
        (&other_instance, true),
    ] {
        let right = source(
            root(21),
            &[right_owner.structs().next().unwrap().clone()],
            &[],
            false,
        )
        .unwrap();
        assert!(left.structs().next().is_none() && right.structs().next().is_none());
        let consumer = source(
            root(22),
            &[],
            &[
                left.functions().next().unwrap().clone(),
                right.functions().next().unwrap().clone(),
            ],
            false,
        );
        assert_eq!(consumer.is_ok(), valid, "{consumer:?}");
        if let Err(error) = consumer {
            assert!(
                error.contains("different certificates for one owner"),
                "{error}"
            );
        }
    }
}

#[test]
fn source_crate_identity_conflicts_even_when_root_hash_and_header_differ() {
    let left = source(root(30), &[], &[], false).unwrap();
    let right = source(
        RustDeclarationId {
            definition_path_hash: 2000,
            ..root(30)
        },
        &[],
        &[],
        false,
    )
    .unwrap();
    let mut registry = CRegistry::new();
    registry
        .import_function(left.functions().next().unwrap().clone())
        .unwrap();
    let before = format!("{registry:?}");
    assert!(
        registry
            .import_function(right.functions().next().unwrap().clone())
            .is_err()
    );
    assert_eq!(format!("{registry:?}"), before);
    let a = source(
        root(31),
        &[],
        &[left.functions().next().unwrap().clone()],
        false,
    )
    .unwrap();
    let b = source(
        root(32),
        &[],
        &[right.functions().next().unwrap().clone()],
        false,
    )
    .unwrap();
    let error = source(
        root(33),
        &[],
        &[
            a.functions().next().unwrap().clone(),
            b.functions().next().unwrap().clone(),
        ],
        false,
    )
    .unwrap_err();
    assert!(
        error.contains("different certificates for one crate"),
        "{error}"
    );
}

#[test]
fn canonical_header_collisions_are_atomic_in_both_registration_orders() {
    let owner = api(0);
    let proof = owner.structs().next().unwrap();
    for file_first in [false, true] {
        let mut registry = CRegistry::new();
        let file = CFileKey {
            path: RelativeOutputPath::new(format!(
                "nested/{}",
                owner.public_header().include_path()
            ))
            .unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        };
        if file_first {
            registry.register_file(file).unwrap();
            let before = format!("{registry:?}");
            assert!(registry.import_struct(proof.clone()).is_err());
            assert_eq!(format!("{registry:?}"), before);
        } else {
            registry.import_struct(proof.clone()).unwrap();
            let before = format!("{registry:?}");
            assert!(registry.register_file(file).is_err());
            assert_eq!(format!("{registry:?}"), before);
        }
    }
}
