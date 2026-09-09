//! Leaf ownership is derived from real declarations, copies, resets and calls.
use super::{
    allocation_fixture::*, heap_fixture::*, owner_leaf_fixture::Leaf, storage_fixture::*, *,
};

#[test]
fn initialized_leaf_claim_move_chain_and_repeated_empty_drop() {
    let leaf = Leaf::new();
    let mut actions = leaf.moved(&leaf.first, &leaf.second);
    actions.push(leaf.f.discard(leaf.f.binary(
        CBinaryOperator::Divide,
        leaf.f.int(21),
        pointed_read(&leaf.f, leaf.f.read(&leaf.second)),
    )));
    actions.extend(leaf.moved(&leaf.second, &leaf.first));
    actions.extend(leaf.drop(&leaf.first));
    actions.extend(leaf.drop(&leaf.first));
    actions.extend(leaf.drop(&leaf.second));
    leaf.check(true, actions, Ok(()));
}

#[test]
fn incomplete_leaf_cannot_be_claimed() {
    for initialized in [false, true] {
        let leaf = Leaf::new();
        let actions = leaf.drop(&leaf.first);
        leaf.check(
            initialized,
            actions,
            if initialized {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}

#[test]
fn move_retires_old_raw_and_typed_borrows_but_preserves_destination() {
    for stale in 0..3 {
        let leaf = Leaf::new();
        let mut actions = leaf.moved(&leaf.first, &leaf.second);
        actions.push(leaf.f.discard(leaf.f.read(match stale {
            0 => &leaf.second,
            1 => &leaf.raw,
            _ => &leaf.temporary,
        })));
        actions.extend(leaf.drop(&leaf.second));
        leaf.check(
            true,
            actions,
            if stale == 0 {
                Ok(())
            } else {
                Err(CSafetyError::UnprovedOwnership)
            },
        );
    }
}

#[test]
fn shallow_alias_and_self_move_are_not_independent_owners() {
    for variant in 0..3 {
        let leaf = Leaf::new();
        let actions = vec![match variant {
            0 => assign(&leaf.f, &leaf.second, leaf.f.read(&leaf.temporary)),
            1 => assign(&leaf.f, &leaf.first, leaf.f.read(&leaf.first)),
            _ => assign(
                &leaf.f,
                &leaf.second,
                restore(&leaf.f, &leaf.allocation, leaf.f.read(&leaf.raw)),
            ),
        }];
        leaf.check(true, actions, Err(CSafetyError::UnprovedOwnership));
    }
}

#[test]
fn move_requires_adjacent_exact_source_reset() {
    for variant in 0..4 {
        let leaf = Leaf::new();
        let mut actions = vec![assign(&leaf.f, &leaf.second, leaf.f.read(&leaf.first))];
        match variant {
            0 => {}
            1 => actions.push(assign(&leaf.f, &leaf.second, null(&leaf.f, &leaf.second))),
            2 => actions.push(leaf.f.discard(leaf.f.int(1))),
            _ => actions.push(leaf.f.discard(leaf.f.read(&leaf.second))),
        }
        leaf.check(true, actions, Err(CSafetyError::UnprovedOwnership));
    }
}

#[test]
fn release_requires_owner_origin_and_adjacent_reset() {
    for variant in 0..4 {
        let leaf = Leaf::new();
        let actions = match variant {
            0 => leaf.drop(&leaf.first),
            1 => vec![release(&leaf.f, leaf.f.read(&leaf.raw))],
            2 => vec![release(&leaf.f, erase(&leaf.f, leaf.f.read(&leaf.first)))],
            _ => vec![
                release(&leaf.f, erase(&leaf.f, leaf.f.read(&leaf.first))),
                assign(&leaf.f, &leaf.second, null(&leaf.f, &leaf.second)),
            ],
        };
        leaf.check(
            true,
            actions,
            if variant == 0 {
                Ok(())
            } else {
                Err(CSafetyError::UnprovedOwnership)
            },
        );
    }
}

#[test]
fn clearing_live_owner_or_returning_early_cannot_hide_obligation() {
    for early in [false, true] {
        let leaf = Leaf::new();
        let actions = vec![if early {
            leaf.f.ast().return_statement(None).unwrap()
        } else {
            assign(&leaf.f, &leaf.first, null(&leaf.f, &leaf.first))
        }];
        leaf.check(true, actions, Err(CSafetyError::UnprovedOwnership));
    }
}

#[test]
fn drop_retires_old_borrows() {
    let leaf = Leaf::new();
    let mut actions = leaf.drop(&leaf.first);
    actions.push(leaf.f.discard(leaf.f.read(&leaf.temporary)));
    leaf.check(true, actions, Err(CSafetyError::UnprovedOwnership));
}

#[test]
fn moved_or_dropped_source_cannot_start_another_move() {
    for moved in [false, true] {
        let leaf = Leaf::new();
        let mut actions = if moved {
            leaf.moved(&leaf.first, &leaf.second)
        } else {
            leaf.drop(&leaf.first)
        };
        if moved {
            actions.extend(leaf.drop(&leaf.second));
        }
        actions.push(assign(&leaf.f, &leaf.second, leaf.f.read(&leaf.first)));
        leaf.check(true, actions, Err(CSafetyError::UnprovedOwnership));
    }
}
