//! Adversarial internal metadata controls. These never authenticate source input.
use super::{Facts, Interner, Key, MAX_OWNERS, MAX_USES, Observation};
use portable_codegen::{RustDeclarationId as Id, RustResultVariantFacts as Variant};
use std::sync::Arc;

fn id(definition_path_hash: u64) -> Id {
    Id {
        crate_id: 7,
        definition_path_hash,
    }
}

fn facts(index: u64) -> Facts {
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
    .unwrap()
}

pub(super) fn run() {
    owner_capacity();
    use_capacity();
    conflicting_facts();
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
        let repeated = exact.intern_facts(original.facts()).unwrap();
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
        facts: originals[0].facts(),
    });
    assert!(!frozen.contains_original(&lookalike));

    let mut over = Interner::new(1).unwrap();
    for original in &originals {
        over.intern_facts(original.facts()).unwrap();
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
        let mut root = original.core_root();
        let mut ok = original.ok();
        let mut err = original.err();
        match role {
            0 => root = id(99),
            1 => ok.variant = id(99),
            2 => err.variant = id(99),
            3 => ok.payload = id(99),
            4 => err.payload = id(99),
            _ => unreachable!(),
        }
        let changed = Facts::new(original.key(), root, ok, err).unwrap();
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
            .map(|(key, value)| (*key, value.facts()))
            .collect();
        if let Some(expected) = &expected {
            assert_eq!(expected, &observed);
        } else {
            expected = Some(observed);
        }
    }
}
