//! Metadata consistency tests; fabricated IDs here never authenticate Rust code.
use portable_codegen::{
    RustCanonicalInstanceFacts as Facts, RustCanonicalInstanceKey as Key,
    RustCanonicalInstanceKind as Kind, RustDeclarationId as Id, RustInstanceDefinitionRole as Role,
    RustInstanceIdentityError as Error, RustResultVariantFacts as Variant,
    TargetPackageOwner as Owner,
};
use std::collections::{BTreeSet, HashSet};

fn id(crate_id: u64, definition_path_hash: u64) -> Id {
    Id {
        crate_id,
        definition_path_hash,
    }
}

fn definitions(crate_id: u64) -> [Id; 7] {
    std::array::from_fn(|index| id(crate_id, index as u64))
}

fn facts(ids: [Id; 7]) -> Result<Facts, Error> {
    Facts::new(
        Key::i32_try_from_int_error_result(ids[1], ids[2])?,
        ids[0],
        Variant {
            variant: ids[3],
            payload: ids[5],
        },
        Variant {
            variant: ids[4],
            payload: ids[6],
        },
    )
}

fn reconstruct(facts: Facts) -> [Id; 7] {
    [
        facts.core_root(),
        facts.key().result_definition(),
        facts.key().error_definition(),
        facts.ok().variant,
        facts.err().variant,
        facts.ok().payload,
        facts.err().payload,
    ]
}

#[test]
fn exact_original_roles_survive_even_at_identity_boundaries() {
    for crate_id in [0, 1, u64::MAX] {
        let mut ids = definitions(crate_id);
        ids[6].definition_path_hash = u64::MAX;
        let observed = facts(ids).unwrap();
        assert_eq!(reconstruct(observed), ids);
        assert_eq!(observed.key().kind(), Kind::I32TryFromIntErrorResult);
    }
}

#[test]
fn key_rejects_cross_crate_and_identical_definition_pairs() {
    for crate_id in [0, 1, u64::MAX] {
        let result = id(crate_id, 7);
        assert_eq!(
            Key::i32_try_from_int_error_result(result, result),
            Err(Error::RepeatedDefinition {
                first: Role::Result,
                second: Role::Error,
            })
        );
        assert_eq!(
            Key::i32_try_from_int_error_result(result, id(crate_id ^ 1, 8)),
            Err(Error::DifferentCrates {
                first: Role::Result,
                second: Role::Error,
            })
        );
    }
}

#[test]
fn every_definition_role_is_joined_to_the_actual_core_anchor() {
    for changed in 0..7 {
        let mut ids = definitions(4);
        ids[changed].crate_id = 5;
        assert!(matches!(facts(ids), Err(Error::DifferentCrates { .. })));
    }
}

#[test]
fn all_twenty_one_duplicate_definition_pairs_are_rejected() {
    let roles = [
        Role::CoreRoot,
        Role::Result,
        Role::Error,
        Role::Ok,
        Role::Err,
        Role::OkPayload,
        Role::ErrPayload,
    ];
    let mut tested = 0;
    for first in 0..7 {
        for second in first + 1..7 {
            let mut ids = definitions(4);
            ids[second] = ids[first];
            assert_eq!(
                facts(ids),
                Err(Error::RepeatedDefinition {
                    first: roles[first],
                    second: roles[second],
                })
            );
            tested += 1;
        }
    }
    assert_eq!(tested, 21);
}

#[test]
fn fact_equality_includes_every_original_role_not_just_the_instance_key() {
    let original = facts(definitions(4)).unwrap();
    for changed in 0..7 {
        let mut ids = definitions(4);
        ids[changed].definition_path_hash = 99;
        // Internally consistent descriptions are not compiler-authenticated.
        let other = facts(ids).unwrap();
        assert_ne!(original, other);
        assert_eq!(original.key() == other.key(), ![1, 2].contains(&changed));
    }
    let mut swapped = definitions(4);
    swapped.swap(3, 4);
    swapped.swap(5, 6);
    let other = facts(swapped).unwrap();
    assert_eq!(original.key(), other.key());
    assert_ne!(original, other, "Ok and Err roles cannot be exchanged");
}

#[test]
fn key_preserves_both_definition_roles_and_the_complete_crate_identity() {
    let original = facts(definitions(4)).unwrap().key();
    let reversed = Key::i32_try_from_int_error_result(
        original.error_definition(),
        original.result_definition(),
    )
    .unwrap();
    let other_crate = facts(definitions(5)).unwrap().key();
    assert_ne!(original, reversed);
    assert_ne!(original, other_crate);
    assert_eq!(BTreeSet::from([original, reversed, other_crate]).len(), 3);
    assert_eq!(HashSet::from([original, reversed, other_crate]).len(), 3);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum FixtureProfile {
    First,
    Second,
}

fn owner(facts: Facts, profile: FixtureProfile) -> Owner<FixtureProfile> {
    Owner::CanonicalInstance {
        instance: facts.key(),
        profile,
    }
}

#[test]
fn source_anchor_is_not_generated_ownership_and_profiles_are_part_of_identity() {
    let first = facts(definitions(4)).unwrap();
    let mut ids = definitions(4);
    ids[2].definition_path_hash = 99;
    let second = facts(ids).unwrap();
    assert_eq!(first.core_root(), second.core_root());
    let owners = [
        Owner::SourceCrate(first.core_root()),
        Owner::SourceCrate(first.key().result_definition()),
        owner(first, FixtureProfile::First),
        owner(first, FixtureProfile::Second),
        owner(second, FixtureProfile::First),
    ];
    assert_eq!(BTreeSet::from(owners).len(), owners.len());
    assert_eq!(HashSet::from(owners).len(), owners.len());
}

#[test]
fn descriptive_owner_order_is_independent_of_encounter_order() {
    let input: Vec<_> = (0..32)
        .map(|index| owner(facts(definitions(index)).unwrap(), FixtureProfile::First))
        .collect();
    let expected: BTreeSet<_> = input.iter().copied().collect();
    for offset in 0..input.len() {
        let mut encounter_order = input.clone();
        encounter_order.rotate_left(offset);
        let permuted: BTreeSet<_> = encounter_order.into_iter().rev().collect();
        assert_eq!(expected, permuted);
    }
    let mut with_consumer = expected.clone();
    with_consumer.insert(Owner::SourceCrate(id(100, 0)));
    assert!(expected.is_subset(&with_consumer));
    assert_eq!(with_consumer.len(), expected.len() + 1);
}

#[test]
fn metadata_is_fixed_size_copy_data_without_caller_owned_allocations() {
    fn assert_copy<T: Copy>() {}
    assert_copy::<Key>();
    assert_copy::<Facts>();
    assert_copy::<Owner<FixtureProfile>>();
    assert!(!std::mem::needs_drop::<Facts>());
    assert!(std::mem::size_of::<Facts>() <= 128);
    assert!(std::mem::size_of::<Owner<FixtureProfile>>() <= 64);
}
