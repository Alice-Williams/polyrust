//! Restores preserve identity and require actual success and sufficient layout.
use super::{
    allocation_fixture::*, heap_fixture::*, numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn fixed_scalar_restore_checks_bytes_and_initialization() {
    for bytes in [1, 3, 4, 16] {
        for initialized in [false, true] {
            let mut f = Fixture::new(&[]);
            let raw = raw(&mut f, "raw");
            let ty = CObjectType::scalar(CScalarType::Int);
            let descriptor = descriptor(&mut f, "object", ty.clone());
            let typed = local(&mut f, pointer_type(ty), "typed");
            let mut actions = vec![];
            if initialized {
                actions.push(f.ast().assign(pointee(&f, &typed), f.int(7)).unwrap());
            }
            actions.push(f.discard(pointed_read(&f, f.read(&typed))));
            let body = guarded(&mut f, &raw, &typed, &descriptor, bytes, actions);
            check(
                &f,
                body,
                if bytes < 4 {
                    Err(CSafetyError::UnprovedAllocationSize)
                } else if !initialized {
                    Err(CSafetyError::UninitializedStorage)
                } else {
                    Ok(())
                },
            );
        }
    }
}

#[test]
fn a_cast_does_not_prove_allocation_success() {
    for actual_allocation in [false, true] {
        let mut f = Fixture::new(&[]);
        let raw = raw(&mut f, "raw");
        let ty = CObjectType::scalar(CScalarType::Int);
        let descriptor = descriptor(&mut f, "object", ty.clone());
        let typed = local(&mut f, pointer_type(ty), "typed");
        let start = if actual_allocation {
            allocate(&f, f.size(4))
        } else {
            null(&f, &raw)
        };
        check(
            &f,
            vec![
                f.declare(&raw, start),
                f.declare(&typed, restore(&f, &descriptor, f.read(&raw))),
            ],
            Err(CSafetyError::NullStorage),
        );
    }
}

#[test]
fn another_descriptor_reuses_initialization_without_retyping_the_allocation() {
    for compatible in [false, true] {
        let mut f = Fixture::new(&[]);
        let raw = raw(&mut f, "raw");
        let descriptor = descriptor(&mut f, "object", CObjectType::scalar(CScalarType::Int));
        let other = super::heap_fixture::descriptor(
            &mut f,
            "alias",
            CObjectType::scalar(if compatible {
                CScalarType::I32
            } else {
                CScalarType::U32
            }),
        );
        let typed = local(
            &mut f,
            pointer_type(descriptor.object_type().clone()),
            "typed",
        );
        let alias = local(&mut f, pointer_type(other.object_type().clone()), "alias");
        let actions = vec![
            f.ast().assign(pointee(&f, &typed), f.int(7)).unwrap(),
            assign(&f, &alias, restore(&f, &other, erase(&f, f.read(&typed)))),
            f.discard(pointed_read(&f, f.read(&alias))),
        ];
        let mut body = guarded(&mut f, &raw, &typed, &descriptor, 8, actions);
        body.insert(0, f.declare(&alias, null(&f, &alias)));
        check(
            &f,
            body,
            if compatible {
                Ok(())
            } else {
                Err(CSafetyError::StorageTypeMismatch)
            },
        );
    }
}

#[test]
fn restoring_again_does_not_initialize_unwritten_storage() {
    let mut f = Fixture::new(&[]);
    let raw = raw(&mut f, "raw");
    let descriptor = descriptor(&mut f, "object", CObjectType::scalar(CScalarType::Int));
    let typed = local(
        &mut f,
        pointer_type(descriptor.object_type().clone()),
        "typed",
    );
    let actions = vec![
        assign(&f, &typed, restore(&f, &descriptor, f.read(&raw))),
        f.discard(pointed_read(&f, f.read(&typed))),
    ];
    let body = guarded(&mut f, &raw, &typed, &descriptor, 4, actions);
    check(&f, body, Err(CSafetyError::UninitializedStorage));
}

#[test]
fn spare_bytes_do_not_turn_a_scalar_into_an_array() {
    for index in [0, 1] {
        let mut f = Fixture::new(&[]);
        let raw = raw(&mut f, "raw");
        let descriptor = descriptor(&mut f, "object", CObjectType::scalar(CScalarType::Int));
        let typed = local(
            &mut f,
            pointer_type(descriptor.object_type().clone()),
            "typed",
        );
        let actions = vec![
            f.ast()
                .assign(pointer_index(&f, f.read(&typed), index), f.int(7))
                .unwrap(),
        ];
        let body = guarded(&mut f, &raw, &typed, &descriptor, 64, actions);
        check(
            &f,
            body,
            if index == 0 {
                Ok(())
            } else {
                Err(CSafetyError::IndexOutOfBounds)
            },
        );
    }
}

#[test]
fn restored_default_base_can_release_and_invalidates_raw_and_typed_copies() {
    for stale in 0..3 {
        let mut f = Fixture::new(&[]);
        let raw = raw(&mut f, "raw");
        let descriptor = descriptor(&mut f, "object", CObjectType::scalar(CScalarType::Int));
        let typed = local(
            &mut f,
            pointer_type(descriptor.object_type().clone()),
            "typed",
        );
        let mut success = vec![
            assign(&f, &typed, restore(&f, &descriptor, f.read(&raw))),
            release(&f, erase(&f, f.read(&typed))),
        ];
        if stale != 0 {
            success.push(f.discard(f.read(if stale == 1 { &raw } else { &typed })));
        }
        let branch = f.branch(nonnull(&f, f.read(&raw)), success, vec![]);
        check(
            &f,
            vec![
                f.declare(&raw, allocate(&f, f.size(4))),
                f.declare(&typed, null(&f, &typed)),
                branch,
            ],
            if stale == 0 {
                Ok(())
            } else {
                Err(CSafetyError::ExpiredStorage)
            },
        );
    }
}
