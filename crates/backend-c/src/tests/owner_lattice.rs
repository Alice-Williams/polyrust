//! Private lattice controls do not create public lifecycle certificates.
use super::*;
use crate::ast::{CScalarType, numeric_fixture::Fixture};

#[test]
fn slot_joins_are_commutative_associative_and_never_settle_unknown_resources() {
    let mut f = Fixture::new(&[]);
    let a = f.local(CScalarType::Int, "a");
    let b = f.local(CScalarType::Int, "b");
    // Artificial roots are only lattice tokens here, not allocation evidence.
    let slots = [
        Slot::Uninitialized,
        Slot::Empty,
        Slot::Moved,
        Slot::Dropped,
        Slot::Unproved,
        Slot::Live(Box::new(Root::Local(a))),
        Slot::Live(Box::new(Root::Local(b))),
    ];
    for left in &slots {
        assert_eq!(left.join(left), *left);
        for right in &slots {
            let joined = left.join(right);
            assert_eq!(joined, right.join(left));
            if !left.settled() || !right.settled() {
                assert!(!joined.settled());
            }
            for third in &slots {
                assert_eq!(joined.join(third), left.join(&right.join(third)));
            }
        }
    }
}

#[test]
fn missing_and_pending_predecessors_cannot_manufacture_empty_or_finished_slots() {
    let mut f = Fixture::new(&[]);
    let local = f.local(CScalarType::Int, "slot");
    let empty = Owners {
        slots: BTreeMap::from([(local.clone(), Slot::Empty)]),
        ..Owners::default()
    };
    let mut pending = empty.clone();
    pending.pending = Some(Transaction::Drop {
        source: local.clone(),
    });
    for (left, right) in [(&empty, &Owners::default()), (&empty, &pending)] {
        let joined = left.join(right);
        assert_eq!(joined, right.join(left));
        assert_eq!(joined.finish(), Err(E::UnprovedOwnership));
    }
    let mut failed = pending.clone();
    failed.fail();
    assert_eq!(failed.pending, pending.pending);
    assert_eq!(failed.slots, pending.slots);
    assert_eq!(failed.finish(), Err(E::UnprovedOwnership));
}

#[test]
fn scope_retirement_never_discards_unsettled_states() {
    let mut f = Fixture::new(&[]);
    let local = f.local(CScalarType::Int, "slot");
    for slot in [
        Slot::Uninitialized,
        Slot::Empty,
        Slot::Moved,
        Slot::Dropped,
        Slot::Unproved,
        Slot::Live(Box::new(Root::Local(local.clone()))),
    ] {
        let mut owners = Owners {
            slots: BTreeMap::from([(local.clone(), slot.clone())]),
            ..Owners::default()
        };
        let before = owners.clone();
        if slot.settled() {
            owners.leave(&f.scope).unwrap();
            assert!(owners.slots.is_empty());
        } else {
            assert_eq!(owners.leave(&f.scope), Err(E::UnprovedOwnership));
            assert_eq!(owners, before);
        }
    }
}
