//! Fabricated metadata checks; these tests cannot authenticate compiler facts.
use portable_codegen::{
    RustCanonicalErrorKindFacts as ErrorFacts, RustCanonicalInstanceFacts as Facts,
    RustCanonicalInstanceKey as Key, RustDeclarationId as Id, RustInstanceIdentityError,
    RustIntegerErrorKind as Kind, RustIntegerErrorVariants as Variants,
    RustResultVariantFacts as Variant,
};
use std::collections::{BTreeSet, HashSet};

fn definitions() -> [Id; 15] {
    std::array::from_fn(|index| Id {
        crate_id: 7,
        definition_path_hash: index as u64,
    })
}

fn facts(ids: [Id; 15]) -> Result<ErrorFacts, RustInstanceIdentityError> {
    let instance = Facts::new(
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
    )?;
    ErrorFacts::new(
        instance,
        ids[7],
        ids[8],
        Variants {
            empty: ids[9],
            invalid_digit: ids[10],
            positive_overflow: ids[11],
            negative_overflow: ids[12],
            zero: ids[13],
            not_a_power_of_two: ids[14],
        },
    )
}

#[test]
fn each_named_original_role_survives() {
    for crate_id in [0, 1, u64::MAX] {
        let ids = definitions().map(|id| Id { crate_id, ..id });
        let observed = facts(ids).unwrap();
        assert_eq!(observed.instance().core_root(), ids[0]);
        assert_eq!(observed.instance().key().result_definition(), ids[1]);
        assert_eq!(observed.instance().key().error_definition(), ids[2]);
        assert_eq!(observed.wrapper_field(), ids[7]);
        assert_eq!(observed.kind_definition(), ids[8]);
        let expected = [
            (Kind::Empty, ids[9]),
            (Kind::InvalidDigit, ids[10]),
            (Kind::PosOverflow, ids[11]),
            (Kind::NegOverflow, ids[12]),
            (Kind::Zero, ids[13]),
            (Kind::NotAPowerOfTwo, ids[14]),
        ];
        for (kind, definition) in expected {
            assert_eq!(observed.variant(kind), definition);
        }
    }
}

#[test]
fn all_cross_crate_and_105_duplicate_pairs_reject() {
    for role in 0..15 {
        let mut ids = definitions();
        ids[role].crate_id ^= 1;
        assert!(matches!(
            facts(ids),
            Err(RustInstanceIdentityError::DifferentCrates { .. })
        ));
    }
    let mut count = 0;
    for left in 0..15 {
        for right in left + 1..15 {
            let mut ids = definitions();
            ids[right] = ids[left];
            assert!(matches!(
                facts(ids),
                Err(RustInstanceIdentityError::RepeatedDefinition { .. })
            ));
            count += 1;
        }
    }
    assert_eq!(count, 105);
}

#[test]
fn same_instance_different_error_facts_remain_distinguishable() {
    let original = facts(definitions()).unwrap();
    let mut descriptions = vec![original];
    for role in 7..15 {
        let mut ids = definitions();
        ids[role].definition_path_hash = u64::MAX;
        let altered = facts(ids).unwrap();
        assert_eq!(original.instance(), altered.instance());
        assert_ne!(original, altered);
        descriptions.push(altered);
    }
    let mut swapped = definitions();
    swapped.swap(11, 12);
    descriptions.push(facts(swapped).unwrap());
    assert_eq!(descriptions.iter().collect::<BTreeSet<_>>().len(), 10);
    assert_eq!(descriptions.iter().collect::<HashSet<_>>().len(), 10);
}

#[test]
fn transport_mapping_is_total_injective_and_rejects_invalid_codes() {
    let expected = [
        (Kind::Empty, 0),
        (Kind::InvalidDigit, 1),
        (Kind::PosOverflow, 2),
        (Kind::NegOverflow, 3),
        (Kind::Zero, 4),
        (Kind::NotAPowerOfTwo, 5),
    ];
    assert_eq!(Kind::ALL, expected.map(|(kind, _)| kind));
    for (kind, code) in expected {
        assert_eq!(kind.transport_code(), code);
        assert_eq!(Kind::from_transport_code(code), Some(kind));
    }
    for code in [i32::MIN, -2, -1, 6, 7, 255, 256, i32::MAX] {
        assert_eq!(Kind::from_transport_code(code), None);
    }
}

#[test]
fn error_metadata_is_fixed_copy_and_allocation_free() {
    fn require_copy<T: Copy>() {}
    require_copy::<ErrorFacts>();
    require_copy::<Variants>();
    require_copy::<Kind>();
    assert!(!std::mem::needs_drop::<ErrorFacts>());
    assert!(std::mem::size_of::<ErrorFacts>() <= 15 * std::mem::size_of::<Id>());
}

#[test]
fn all_six_encoded_states_survive_a_checked_typed_fixture_boundary() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Packet {
        Success(i32),
        Error(Kind),
    }
    fn decode(success: bool, value: i32) -> Option<Packet> {
        if success {
            Some(Packet::Success(value))
        } else {
            Kind::from_transport_code(value).map(Packet::Error)
        }
    }
    fn forward(packet: Packet) -> Packet {
        packet
    }
    fn encode(packet: Packet) -> (bool, i32) {
        match packet {
            Packet::Success(value) => (true, value),
            Packet::Error(kind) => (false, kind.transport_code()),
        }
    }
    let expected = [
        Kind::Empty,
        Kind::InvalidDigit,
        Kind::PosOverflow,
        Kind::NegOverflow,
        Kind::Zero,
        Kind::NotAPowerOfTwo,
    ];
    let mut collapse_failures = 0;
    let mut swap_failures = 0;
    for (code, kind) in (0..6).zip(expected) {
        let input = decode(false, code).unwrap();
        assert_eq!(input, Packet::Error(kind));
        assert_eq!(encode(forward(input)), (false, code));
        let collapsed = Packet::Error(Kind::Empty);
        let swapped = match kind {
            Kind::PosOverflow => Packet::Error(Kind::NegOverflow),
            Kind::NegOverflow => Packet::Error(Kind::PosOverflow),
            _ => input,
        };
        collapse_failures += usize::from(encode(collapsed) != (false, code));
        swap_failures += usize::from(encode(swapped) != (false, code));
    }
    assert_eq!((collapse_failures, swap_failures), (5, 2));
    for invalid in [i32::MIN, -1, 6, i32::MAX] {
        assert_eq!(decode(false, invalid), None);
        assert_eq!(encode(decode(true, invalid).unwrap()), (true, invalid));
    }
    // This fixture proves the code vocabulary boundary, not C/Java rendering.
}
