//! Canonical type-owner transport, not Rust compiler Result source admission.
mod driver;
#[path = "../canonical_dependencies/fixture.rs"]
mod fixture;
mod native;
mod origins;
mod package;
use package::*;
use portable_backend_c::dialect::*;

#[test]
fn independent_producers_share_all_six_error_states_and_success_payloads() {
    let owner = fixture::api(0);
    let proof = owner.structs().next().unwrap();
    let a = producer(110, proof, Fault::None);
    let b = producer(111, proof, Fault::None);
    let consumer = package(
        112,
        proof,
        &[],
        &[Operation::Compose {
            inner: function(&a, "construct").into(),
            outer: function(&b, "forward").into(),
            duplicate: false,
        }],
        false,
    );
    let backward = package(
        113,
        proof,
        &[],
        &[Operation::Compose {
            inner: function(&b, "construct").into(),
            outer: function(&a, "forward").into(),
            duplicate: false,
        }],
        false,
    );
    for api in [&a, &b, &consumer, &backward] {
        assert_eq!(api.structure(proof.record()), Some(proof));
        assert_eq!(api.structs().count(), 1);
        assert_eq!(api.structs().next().unwrap().members(), proof.members());
    }
    native::prove(&owner, &a, &b, &consumer, &backward);
}

#[test]
fn import_order_and_unrelated_owners_do_not_rename_or_redefine_the_original() {
    let owner = fixture::api(0);
    let other = fixture::api(100);
    let proof = owner.structs().next().unwrap();
    let original = native::texts(&owner);
    let a = producer(120, proof, Fault::None);
    let b = producer(121, proof, Fault::None);
    let build = |extra: &[CDependencyStruct], reverse, backward| {
        package(
            if backward { 123 } else { 122 },
            proof,
            extra,
            &[Operation::Compose {
                inner: function(if backward { &b } else { &a }, "construct").into(),
                outer: function(if backward { &a } else { &b }, "forward").into(),
                duplicate: false,
            }],
            reverse,
        )
    };
    for backward in [false, true] {
        let baseline = native::texts(&build(&[], false, backward));
        for reverse in [false, true] {
            let extra = [other.structs().next().unwrap().clone()];
            let consumer = build(&extra, reverse, backward);
            assert_eq!(native::texts(&consumer), baseline);
            assert_eq!(native::texts(&owner), original);
            assert_eq!(consumer.dependencies().unwrap().len(), 4);
            for (_, text) in native::texts(&consumer) {
                assert!(!text.contains(&format!("struct {} {{", proof.symbol().as_str())));
            }
        }
    }
}
