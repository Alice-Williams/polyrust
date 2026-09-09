//! Type bindings are must-compatible and cannot be manufactured by branch syntax.
use super::{
    allocation_fixture::*, contextual_reconstruction::key, heap_fixture::*,
    numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn incompatible_branch_bindings_do_not_become_unbound_after_the_join() {
    for compatible in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let raw = raw(&mut f, "raw");
        let ty = CObjectType::scalar(CScalarType::Int);
        let object = descriptor(&mut f, "object", ty.clone());
        let other = descriptor(
            &mut f,
            "other",
            if compatible {
                ty
            } else {
                CObjectType::scalar(CScalarType::U32)
            },
        );
        let live = require_live(&mut f, &raw, vec![]);
        let branch = f.branch(
            f.input(0),
            vec![f.discard(restore(&f, &object, f.read(&raw)))],
            vec![f.discard(restore(&f, &other, f.read(&raw)))],
        );
        check(
            &f,
            vec![
                f.declare(&raw, allocate(&f, f.size(4))),
                live,
                branch,
                f.discard(restore(&f, &object, f.read(&raw))),
                release(&f, f.read(&raw)),
            ],
            if compatible {
                Ok(())
            } else {
                Err(CSafetyError::StorageTypeMismatch)
            },
        );
    }
}

#[test]
fn nested_restores_reuse_a_binding_but_never_establish_one() {
    for bound in [false, true] {
        for selected in [false, true] {
            let mut f = Fixture::new(&[]);
            let raw = raw(&mut f, "raw");
            let ty = CObjectType::scalar(CScalarType::Int);
            let object = descriptor(&mut f, "object", ty.clone());
            let pointer = local(&mut f, pointer_type(ty), "pointer");
            let live = require_live(&mut f, &raw, vec![]);
            let mut body = vec![
                f.declare(&raw, allocate(&f, f.size(4))),
                live,
                f.declare(&pointer, null(&f, &pointer)),
            ];
            if bound {
                body.push(f.discard(restore(&f, &object, f.read(&raw))));
            }
            let conditional = f
                .values()
                .conditional(
                    f.boolean(f.int(i32::from(selected))),
                    restore(&f, &object, f.read(&raw)),
                    null(&f, &pointer),
                )
                .unwrap();
            body.extend([assign(&f, &pointer, conditional), release(&f, f.read(&raw))]);
            check(
                &f,
                body,
                if selected && !bound {
                    Err(CSafetyError::UnprovedAllocation)
                } else {
                    Ok(())
                },
            );
        }
    }
}

#[test]
fn a_custom_descriptor_cannot_claim_a_default_allocation() {
    let mut f = Fixture::new(&[]);
    let raw = raw(&mut f, "raw");
    let allocator = f.local(CScalarType::Int, "allocator");
    let object = f
        .registry
        .register_allocation(
            &f.scope,
            key("object"),
            CObjectType::scalar(CScalarType::Int),
            CAllocatorSource::Local(allocator.clone()),
        )
        .unwrap();
    let live = require_live(&mut f, &raw, vec![]);
    check(
        &f,
        vec![
            f.declare(&allocator, f.int(0)),
            f.declare(&raw, allocate(&f, f.size(4))),
            live,
            f.discard(restore(&f, &object, f.read(&raw))),
            release(&f, f.read(&raw)),
        ],
        Err(CSafetyError::UnprovedAllocation),
    );
}

#[test]
fn heap_initialization_cannot_launder_numeric_allocation_history() {
    for wrapped in [false, true] {
        let mut f = Fixture::new(&[]);
        let raw = raw(&mut f, "raw");
        let next = raw_pointer(&mut f);
        let object = descriptor(&mut f, "size", CObjectType::scalar(CScalarType::Size));
        let pointer = local(
            &mut f,
            pointer_type(object.object_type().clone()),
            "pointer",
        );
        let live = require_live(&mut f, &raw, vec![]);
        let size = if wrapped {
            f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(1))
        } else {
            f.size(4)
        };
        check(
            &f,
            vec![
                f.declare(&raw, allocate(&f, f.size(8))),
                live,
                f.declare(&pointer, restore(&f, &object, f.read(&raw))),
                f.ast().assign(pointee(&f, &pointer), size).unwrap(),
                f.declare(&next, allocate(&f, pointed_read(&f, f.read(&pointer)))),
                release(&f, f.read(&next)),
                release(&f, f.read(&raw)),
            ],
            if wrapped {
                Err(CSafetyError::UnprovedSizeArithmetic)
            } else {
                Ok(())
            },
        );
    }
}
fn raw_pointer(f: &mut Fixture) -> CLocalRef {
    raw(f, "next")
}
