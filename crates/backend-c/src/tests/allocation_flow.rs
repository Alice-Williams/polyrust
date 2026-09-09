//! Positive raw ownership flow and rejected loss/release/alias mutations.
use super::{allocation_fixture::*, numeric_fixture::Fixture, storage_fixture::check, *};

#[test]
fn positive_checked_bytes_and_null_safe_default_release() {
    for bytes in [0, 1, 16, u64::MAX] {
        let mut f = Fixture::new(&[]);
        let pointer = raw(&mut f, "pointer");
        let mut body = vec![f.declare(&pointer, allocate(&f, f.size(bytes)))];
        if bytes != 0 {
            body.push(release(&f, f.read(&pointer)));
        }
        check(
            &f,
            body,
            if bytes == 0 {
                Err(CSafetyError::UnprovedAllocationSize)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn runtime_allocation_requires_a_dominating_positive_guard() {
    for guarded in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let pointer = raw(&mut f, "pointer");
        let body = vec![f.declare(&pointer, allocate(&f, f.input(0)))];
        // The local belongs to the function scope, so declaration remains there;
        // guarded assignment is used inside the child scope.
        let body = if guarded {
            let assignment = f
                .ast()
                .assign(
                    f.values().local(pointer.clone()).unwrap(),
                    allocate(&f, f.input(0)),
                )
                .unwrap();
            let branch = f.branch(
                f.compare(CBinaryOperator::Greater, f.input(0), f.size(0)),
                vec![assignment, release(&f, f.read(&pointer))],
                vec![],
            );
            vec![f.declare(&pointer, null(&f, &pointer)), branch]
        } else {
            body
        };
        check(
            &f,
            body,
            if guarded {
                Ok(())
            } else {
                Err(CSafetyError::UnprovedAllocationSize)
            },
        );
    }
}

#[test]
fn null_and_success_branches_account_for_the_same_allocation() {
    for release_success in [false, true] {
        let mut f = Fixture::new(&[]);
        let pointer = raw(&mut f, "pointer");
        let success = if release_success {
            vec![release(&f, f.read(&pointer))]
        } else {
            vec![]
        };
        let branch = f.branch(
            nonnull(&f, f.read(&pointer)),
            success,
            vec![f.ast().return_statement(None).unwrap()],
        );
        check(
            &f,
            vec![f.declare(&pointer, allocate(&f, f.size(8))), branch],
            if release_success {
                Ok(())
            } else {
                Err(CSafetyError::UnreleasedAllocation)
            },
        );
    }
}

#[test]
fn copied_base_can_release_but_every_retained_alias_expires() {
    for after in 0..3 {
        let mut f = Fixture::new(&[]);
        let pointer = raw(&mut f, "pointer");
        let copy = raw(&mut f, "copy");
        let mut body = vec![
            f.declare(&pointer, allocate(&f, f.size(8))),
            f.declare(&copy, f.read(&pointer)),
            release(&f, f.read(&copy)),
        ];
        match after {
            1 => body.push(f.discard(f.read(&pointer))),
            2 => body.push(release(&f, f.read(&copy))),
            _ => {}
        }
        check(
            &f,
            body,
            if after == 0 {
                Ok(())
            } else {
                Err(CSafetyError::ExpiredStorage)
            },
        );
    }
}

#[test]
fn overwriting_a_copy_does_not_change_the_original_allocation_outcome() {
    let mut f = Fixture::new(&[]);
    let pointer = raw(&mut f, "pointer");
    let copy = raw(&mut f, "copy");
    check(
        &f,
        vec![
            f.declare(&pointer, allocate(&f, f.size(8))),
            f.declare(&copy, f.read(&pointer)),
            f.ast()
                .assign(f.values().local(copy.clone()).unwrap(), null(&f, &copy))
                .unwrap(),
            release(&f, f.read(&copy)),
            release(&f, f.read(&pointer)),
        ],
        Ok(()),
    );
}

#[test]
fn losing_or_returning_before_releasing_a_possible_allocation_is_rejected() {
    for path in 0..3 {
        let mut f = Fixture::new(&[]);
        let pointer = raw(&mut f, "pointer");
        let mut body = vec![f.declare(&pointer, allocate(&f, f.size(8)))];
        match path {
            0 => body.push(
                f.ast()
                    .assign(
                        f.values().local(pointer.clone()).unwrap(),
                        null(&f, &pointer),
                    )
                    .unwrap(),
            ),
            1 => body.push(f.ast().return_statement(None).unwrap()),
            _ => {}
        }
        check(&f, body, Err(CSafetyError::UnreleasedAllocation));
    }
}

#[test]
fn release_of_an_automatic_address_is_not_default_allocator_release() {
    let mut f = Fixture::new(&[]);
    let value = f.local(CScalarType::Int, "value");
    let pointer = f
        .values()
        .object_to_void(
            CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified)),
            f.values()
                .address_of(f.values().local(value.clone()).unwrap())
                .unwrap(),
        )
        .unwrap();
    check(
        &f,
        vec![f.declare(&value, f.int(1)), release(&f, pointer)],
        Err(CSafetyError::InvalidAllocationRelease),
    );
}

#[test]
fn a_proved_null_result_may_be_released_repeatedly_without_releasing_storage() {
    let mut f = Fixture::new(&[]);
    let pointer = raw(&mut f, "pointer");
    let branch = f.branch(
        nonnull(&f, f.read(&pointer)),
        vec![release(&f, f.read(&pointer))],
        vec![release(&f, f.read(&pointer)), release(&f, f.read(&pointer))],
    );
    check(
        &f,
        vec![f.declare(&pointer, allocate(&f, f.size(8))), branch],
        Ok(()),
    );
}
