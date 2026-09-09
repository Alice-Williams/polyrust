//! Real branches, crossed scopes and slot aliases cannot erase owner obligations.
use super::{
    allocation_fixture::*, contextual_reconstruction::key, heap_fixture::*,
    owner_leaf_fixture::Leaf, storage_fixture::*, *,
};

#[test]
fn both_runtime_branches_must_discharge_the_owner() {
    for both in [false, true] {
        let mut leaf = Leaf::new();
        let yes = leaf.drop(&leaf.first);
        let no = if both { leaf.drop(&leaf.first) } else { vec![] };
        let branch = leaf.f.branch(leaf.f.input(0), yes, no);
        leaf.check(
            true,
            vec![branch],
            if both {
                Ok(())
            } else {
                Err(CSafetyError::UnprovedOwnership)
            },
        );
    }
}

#[test]
fn control_choice_between_copy_and_reset_is_not_an_atomic_move() {
    let mut leaf = Leaf::new();
    let reset = assign(&leaf.f, &leaf.first, null(&leaf.f, &leaf.first));
    let branch = leaf
        .f
        .branch(leaf.f.input(0), vec![reset.clone()], vec![reset]);
    let mut actions = vec![
        assign(&leaf.f, &leaf.second, leaf.f.read(&leaf.first)),
        branch,
    ];
    actions.extend(leaf.drop(&leaf.second));
    leaf.check(true, actions, Err(CSafetyError::UnprovedOwnership));
}

#[test]
fn move_reset_cannot_enter_a_descendant_scope() {
    transaction_reset_scope(false);
}

#[test]
fn release_reset_cannot_enter_a_descendant_scope() {
    transaction_reset_scope(true);
}

fn transaction_reset_scope(drop: bool) {
    for nested in [false, true] {
        let mut leaf = Leaf::new();
        let mut actions = if drop {
            leaf.drop(&leaf.first)
        } else {
            leaf.moved(&leaf.first, &leaf.second)
        };
        if nested {
            let reset = actions.pop().unwrap();
            let child = leaf
                .f
                .registry
                .register_scope(&leaf.f.function, Some(&leaf.f.scope), key("reset_scope"))
                .unwrap();
            let block = leaf.f.ast().block(child, vec![reset]).unwrap();
            actions.push(leaf.f.ast().nested_block(block).unwrap());
        }
        if !drop {
            actions.extend(leaf.drop(&leaf.second));
        }
        leaf.check(
            true,
            actions,
            if nested {
                Err(CSafetyError::UnprovedOwnership)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn cleanup_jump_must_not_bypass_owner_drop() {
    for bypass in [false, true] {
        let mut leaf = Leaf::new();
        let label = leaf
            .f
            .registry
            .register_cleanup_exit(&leaf.f.scope, key("done"))
            .unwrap();
        let mut actions = if bypass {
            vec![]
        } else {
            leaf.drop(&leaf.first)
        };
        actions.push(leaf.f.ast().cleanup_jump(label.clone()).unwrap());
        let mut body = leaf.body(true, actions);
        body.push(
            leaf.f
                .ast()
                .label(label, leaf.f.ast().return_statement(None).unwrap())
                .unwrap(),
        );
        check(
            &leaf.f,
            body,
            if bypass {
                Err(CSafetyError::UnprovedOwnership)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn crossing_child_scope_accounts_for_its_local_owner() {
    for drop in [false, true] {
        let mut leaf = Leaf::new();
        let child = leaf
            .f
            .registry
            .register_scope(&leaf.f.function, Some(&leaf.f.scope), key("child"))
            .unwrap();
        let local = leaf
            .f
            .registry
            .register_local(&child, key("child_owner"), leaf.first.ty().clone())
            .unwrap();
        leaf.f.registry.register_local_owner(&local).unwrap();
        let mut inner = vec![leaf.f.declare(&local, null(&leaf.f, &local))];
        inner.extend(leaf.moved(&leaf.first, &local));
        if drop {
            inner.extend(leaf.drop(&local));
        }
        let block = leaf
            .f
            .ast()
            .nested_block(leaf.f.ast().block(child, inner).unwrap())
            .unwrap();
        let guard = require_live(&mut leaf.f, &leaf.raw, vec![]);
        let mut body = vec![
            leaf.f.declare(&leaf.raw, allocate(&leaf.f, leaf.f.size(4))),
            guard,
            leaf.f.declare(
                &leaf.temporary,
                restore(&leaf.f, &leaf.allocation, leaf.f.read(&leaf.raw)),
            ),
            leaf.f.declare(&leaf.first, null(&leaf.f, &leaf.first)),
            leaf.f.declare(&leaf.second, null(&leaf.f, &leaf.second)),
            leaf.f
                .ast()
                .assign(pointee(&leaf.f, &leaf.temporary), leaf.f.int(7))
                .unwrap(),
            assign(&leaf.f, &leaf.first, leaf.f.read(&leaf.temporary)),
            block,
        ];
        body.extend(leaf.drop(&leaf.first));
        check(
            &leaf.f,
            body,
            if drop {
                Ok(())
            } else {
                Err(CSafetyError::UnprovedOwnership)
            },
        );
    }
}

#[test]
fn writing_through_an_alias_to_the_owner_slot_cannot_clear_it() {
    let mut leaf = Leaf::new();
    let alias = local(
        &mut leaf.f,
        pointer_type(leaf.first.ty().clone()),
        "slot_address",
    );
    let indirect = leaf
        .f
        .ast()
        .assign(pointee(&leaf.f, &alias), null(&leaf.f, &leaf.first))
        .unwrap();
    let mut body = leaf.body(true, vec![indirect]);
    body.insert(
        4,
        leaf.f.declare(
            &alias,
            address(&leaf.f, leaf.f.values().local(leaf.first.clone()).unwrap()),
        ),
    );
    check(&leaf.f, body, Err(CSafetyError::UnprovedOwnership));
}
