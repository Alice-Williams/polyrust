//! Adversarial internal metadata controls. These never authenticate source input.
use super::{ErrorFacts, Facts, Interner, Key, MAX_OWNERS, MAX_USES, Observation};
use portable_codegen::{
    RustDeclarationId as Id, RustIntegerErrorKind as Kind, RustIntegerErrorVariants as Variants,
    RustResultVariantFacts as Variant,
};
use std::sync::Arc;

fn id(definition_path_hash: u64) -> Id {
    Id {
        crate_id: 7,
        definition_path_hash,
    }
}

fn facts(index: u64) -> ErrorFacts {
    error_facts(
        Facts::new(
            Key::i32_try_from_int_error_result(id(1), id(100 + index)).unwrap(),
            id(0),
            Variant {
                variant: id(3),
                payload: id(5),
            },
            Variant {
                variant: id(4),
                payload: id(6),
            },
        )
        .unwrap(),
    )
}

fn error_facts(instance: Facts) -> ErrorFacts {
    ErrorFacts::new(
        instance,
        id(70),
        id(71),
        Variants {
            empty: id(72),
            invalid_digit: id(73),
            positive_overflow: id(74),
            negative_overflow: id(75),
            zero: id(76),
            not_a_power_of_two: id(77),
        },
    )
    .unwrap()
}

pub(super) fn run() {
    owner_capacity();
    use_capacity();
    conflicting_facts();
    conflicting_error_facts();
    permutations();
}

fn owner_capacity() {
    assert!(Interner::new(0).is_err());
    assert!(Interner::new(MAX_OWNERS + 1).is_err());
    let mut empty_capacity = Interner::new(MAX_OWNERS).unwrap();
    assert!(empty_capacity.intern_facts(facts(0)).is_err());
    assert!(empty_capacity.freeze().is_err());

    let mut exact = Interner::new(1).unwrap();
    let mut originals = Vec::new();
    for index in 0..MAX_OWNERS - 1 {
        originals.push(exact.intern_facts(facts(index as u64)).unwrap());
    }
    for original in &originals {
        let repeated = exact.intern_facts(original.error_facts()).unwrap();
        assert!(Arc::ptr_eq(original, &repeated));
    }
    let frozen = exact.freeze().unwrap();
    assert_eq!(frozen.owners().count(), MAX_OWNERS - 1);
    assert!(
        originals
            .iter()
            .all(|original| frozen.contains_original(original))
    );
    let lookalike = Arc::new(Observation {
        facts: originals[0].error_facts(),
    });
    assert!(!frozen.contains_original(&lookalike));

    let mut over = Interner::new(1).unwrap();
    for original in &originals {
        over.intern_facts(original.error_facts()).unwrap();
    }
    assert_eq!(
        over.intern_facts(facts(MAX_OWNERS as u64)).err().unwrap(),
        "instance graph total owner budget exceeded"
    );
    assert!(over.freeze().is_err());
}

fn use_capacity() {
    let mut exact = Interner::new(1).unwrap();
    let first = exact.intern_facts(facts(0)).unwrap();
    for _ in 1..MAX_USES {
        assert!(Arc::ptr_eq(&first, &exact.intern_facts(facts(0)).unwrap()));
    }
    let frozen = exact.freeze().unwrap();
    assert_eq!(frozen.uses(), MAX_USES);
    assert_eq!(frozen.owners().count(), 1);

    let mut over = Interner::new(1).unwrap();
    for _ in 0..MAX_USES {
        over.intern_facts(facts(0)).unwrap();
    }
    assert_eq!(
        over.intern_facts(facts(0)).err().unwrap(),
        "instance graph use budget exceeded"
    );
    assert!(over.freeze().is_err());
}

fn conflicting_facts() {
    let original = facts(0);
    for role in 0..5 {
        let mut root = original.instance().core_root();
        let mut ok = original.instance().ok();
        let mut err = original.instance().err();
        match role {
            0 => root = id(99),
            1 => ok.variant = id(99),
            2 => err.variant = id(99),
            3 => ok.payload = id(99),
            4 => err.payload = id(99),
            _ => unreachable!(),
        }
        let changed = error_facts(Facts::new(original.instance().key(), root, ok, err).unwrap());
        let mut interner = Interner::new(3).unwrap();
        interner.intern_facts(original).unwrap();
        assert_eq!(
            interner.intern_facts(changed).err().unwrap(),
            "instance graph original facts conflict"
        );
        // A caller cannot recover a publishable prefix by ignoring the error.
        assert!(interner.intern_facts(original).is_err());
        assert!(interner.freeze().is_err());
    }
}

fn permutations() {
    let values: Vec<_> = (0..16).map(facts).collect();
    let mut expected = None;
    for offset in 0..values.len() {
        let mut order = values.clone();
        order.rotate_left(offset);
        order.reverse();
        let mut interner = Interner::new(3 + offset).unwrap();
        for value in order {
            interner.intern_facts(value).unwrap();
        }
        let frozen = interner.freeze().unwrap();
        let observed: Vec<_> = frozen
            .owners()
            .map(|(key, value)| (*key, value.error_facts()))
            .collect();
        if let Some(expected) = &expected {
            assert_eq!(expected, &observed);
        } else {
            expected = Some(observed);
        }
    }
}

fn conflicting_error_facts() {
    let original = facts(0);
    for role in 0..8 {
        let mut ids = [
            original.wrapper_field(),
            original.kind_definition(),
            original.variant(Kind::Empty),
            original.variant(Kind::InvalidDigit),
            original.variant(Kind::PosOverflow),
            original.variant(Kind::NegOverflow),
            original.variant(Kind::Zero),
            original.variant(Kind::NotAPowerOfTwo),
        ];
        ids[role] = id(99);
        let changed = ErrorFacts::new(
            original.instance(),
            ids[0],
            ids[1],
            Variants {
                empty: ids[2],
                invalid_digit: ids[3],
                positive_overflow: ids[4],
                negative_overflow: ids[5],
                zero: ids[6],
                not_a_power_of_two: ids[7],
            },
        )
        .unwrap();
        let mut interner = Interner::new(3).unwrap();
        interner.intern_facts(original).unwrap();
        assert_eq!(
            interner.intern_facts(changed).err().unwrap(),
            "instance graph original facts conflict"
        );
        assert!(interner.freeze().is_err());
    }
}
