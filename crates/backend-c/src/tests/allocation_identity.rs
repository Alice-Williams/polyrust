//! Equal call syntax, byte arithmetic and escaping handles remain distinct.
use super::{
    allocation_fixture::*,
    contextual_reconstruction::key,
    numeric_fixture::Fixture,
    storage_fixture::{aggregate_source, check, local},
    *,
};

#[test]
fn identical_cloned_call_syntax_creates_two_distinct_allocation_obligations() {
    for release_both in [false, true] {
        let mut f = Fixture::new(&[]);
        let a = raw(&mut f, "a");
        let b = raw(&mut f, "b");
        let call = allocate(&f, f.size(8));
        let mut body = vec![
            f.declare(&a, call.clone()),
            f.declare(&b, call),
            release(&f, f.read(&a)),
        ];
        if release_both {
            body.push(release(&f, f.read(&b)));
        }
        check(
            &f,
            body,
            if release_both {
                Ok(())
            } else {
                Err(CSafetyError::UnreleasedAllocation)
            },
        );
    }
}

#[test]
fn wrapped_bytes_cannot_be_laundered_into_an_allocation_request() {
    let f = Fixture::new(&[CScalarType::Size]);
    let wrapped = f.binary(CBinaryOperator::Add, f.input(0), f.size(1));
    check(
        &f,
        vec![f.discard(allocate(&f, wrapped))],
        Err(CSafetyError::UnprovedSizeArithmetic),
    );
}

#[test]
fn release_expires_pointer_copies_nested_in_aggregate_storage() {
    for stale in [false, true] {
        let mut f = Fixture::new(&[]);
        let pointer = raw(&mut f, "pointer");
        let tag = f
            .registry
            .declare_struct(&f.file, key("Container"))
            .unwrap();
        let owner = CAggregateRef::Struct(tag.clone());
        let field = f
            .registry
            .register_member(&owner, key("pointer"), pointer.ty().clone())
            .unwrap();
        f.registry
            .define_aggregate(&owner, vec![field.clone()])
            .unwrap();
        let value = local(&mut f, CObjectType::structure(tag), "container");
        let place = f
            .values()
            .member(f.values().local(value.clone()).unwrap(), field)
            .unwrap();
        let mut body = vec![
            f.declare(&pointer, allocate(&f, f.size(8))),
            f.ast()
                .declare(
                    value.clone(),
                    Some(f.values().zero_initializer(value.ty().clone()).unwrap()),
                )
                .unwrap(),
            f.ast().assign(place.clone(), f.read(&pointer)).unwrap(),
            release(&f, f.values().read(place.clone()).unwrap()),
        ];
        if stale {
            body.push(f.discard(f.read(&value)));
        }
        let source = aggregate_source(&f, owner, body);
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            if stale {
                Err(CSafetyError::ExpiredStorage)
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn raw_allocations_cannot_escape_through_global_slots_or_aggregate_copies() {
    for aggregate in [false, true] {
        let mut f = Fixture::new(&[]);
        let pointer = raw(&mut f, "pointer");
        let tag = f
            .registry
            .declare_struct(&f.file, key("Container"))
            .unwrap();
        let owner = CAggregateRef::Struct(tag.clone());
        let field = f
            .registry
            .register_member(&owner, key("pointer"), pointer.ty().clone())
            .unwrap();
        f.registry
            .define_aggregate(&owner, vec![field.clone()])
            .unwrap();
        let value = local(&mut f, CObjectType::structure(tag), "container");
        let output_type = if aggregate {
            value.ty().clone()
        } else {
            pointer.ty().clone()
        };
        let output = f
            .registry
            .register_object(&f.file, key("output"), output_type)
            .unwrap();
        let place = f
            .values()
            .member(f.values().local(value.clone()).unwrap(), field)
            .unwrap();
        let mut source = aggregate_source(
            &f,
            owner,
            vec![
                f.declare(&pointer, allocate(&f, f.size(8))),
                f.ast()
                    .declare(
                        value.clone(),
                        Some(f.values().zero_initializer(value.ty().clone()).unwrap()),
                    )
                    .unwrap(),
                f.ast().assign(place, f.read(&pointer)).unwrap(),
                f.ast()
                    .assign(
                        f.values().global(output.clone()).unwrap(),
                        if aggregate {
                            f.read(&value)
                        } else {
                            f.read(&pointer)
                        },
                    )
                    .unwrap(),
                release(&f, f.read(&pointer)),
            ],
        );
        source.items.push(CFileItem::Definition(
            CDeclarations::new(&f.registry, f.file.clone())
                .unwrap()
                .object_definition(
                    output.clone(),
                    CLinkage::Internal,
                    f.values().zero_initializer(output.ty().clone()).unwrap(),
                )
                .unwrap(),
        ));
        f.registry
            .check_numeric_flow(std::slice::from_ref(&source))
            .unwrap();
        assert!(
            f.registry.check_storage_paths(&[source]).is_err(),
            "aggregate={aggregate}"
        );
    }
}
