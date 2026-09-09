//! Complete fixed representations transfer; partial or pointer-bearing ones do not.
use super::{
    allocation_fixture::*, contextual_reconstruction::key, heap_fixture::*,
    numeric_fixture::Fixture, owner_leaf_fixture::Leaf, storage_fixture::*, *,
};

#[test]
fn fixed_array_owner_requires_every_element_before_transfer() {
    for complete in [false, true] {
        let ty = CObjectType::array(
            CObjectType::scalar(CScalarType::Int),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let mut leaf = Leaf::with_type(Fixture::new(&[]), ty);
        let construction = (0..if complete { 2 } else { 1 })
            .map(|i| {
                let place = leaf
                    .f
                    .values()
                    .index(
                        CIndexBase::Array(Box::new(pointee(&leaf.f, &leaf.temporary))),
                        leaf.f.size(i),
                    )
                    .unwrap();
                leaf.f.ast().assign(place, leaf.f.int(7)).unwrap()
            })
            .collect();
        let mut actions = leaf.moved(&leaf.first, &leaf.second);
        actions.extend(leaf.drop(&leaf.second));
        let body = leaf.body_with(8, construction, actions);
        check(
            &leaf.f,
            body,
            if complete {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}

#[test]
fn record_and_active_union_payloads_need_complete_pointer_free_construction() {
    for union in [false, true] {
        for complete in [false, true] {
            let mut f = Fixture::new(&[]);
            let aggregate = if union {
                CAggregateRef::Union(f.registry.declare_union(&f.file, key("Payload")).unwrap())
            } else {
                CAggregateRef::Struct(f.registry.declare_struct(&f.file, key("Payload")).unwrap())
            };
            let members = ["first", "second"].map(|name| {
                f.registry
                    .register_member(&aggregate, key(name), CObjectType::scalar(CScalarType::Int))
                    .unwrap()
            });
            f.registry
                .define_aggregate(&aggregate, members.to_vec())
                .unwrap();
            let ty = match &aggregate {
                CAggregateRef::Struct(tag) => CObjectType::structure(tag.clone()),
                CAggregateRef::Union(tag) => CObjectType::union(tag.clone()),
            };
            let mut leaf = Leaf::with_type(f, ty);
            let count = if union {
                usize::from(complete)
            } else if complete {
                2
            } else {
                1
            };
            let construction = members
                .iter()
                .take(count)
                .map(|member| {
                    let place = leaf
                        .f
                        .values()
                        .member(pointee(&leaf.f, &leaf.temporary), member.clone())
                        .unwrap();
                    leaf.f.ast().assign(place, leaf.f.int(7)).unwrap()
                })
                .collect();
            let mut actions = leaf.moved(&leaf.first, &leaf.second);
            actions.extend(leaf.drop(&leaf.second));
            let body = leaf.body_with(8, construction, actions);
            let source = aggregate_source(&leaf.f, aggregate, body);
            assert_eq!(
                leaf.f.registry.check_storage_paths(&[source]),
                if complete {
                    Ok(())
                } else {
                    Err(CSafetyError::UninitializedStorage)
                }
            );
        }
    }
}

#[test]
fn inactive_pointer_child_is_not_a_proved_leaf_owner() {
    let mut f = Fixture::new(&[]);
    let aggregate =
        CAggregateRef::Union(f.registry.declare_union(&f.file, key("Payload")).unwrap());
    let scalar = f
        .registry
        .register_member(
            &aggregate,
            key("number"),
            CObjectType::scalar(CScalarType::Int),
        )
        .unwrap();
    let child = f
        .registry
        .register_member(
            &aggregate,
            key("child"),
            pointer_type(CObjectType::scalar(CScalarType::Int)),
        )
        .unwrap();
    f.registry
        .define_aggregate(&aggregate, vec![scalar.clone(), child])
        .unwrap();
    let CAggregateRef::Union(tag) = &aggregate else {
        unreachable!()
    };
    let mut leaf = Leaf::with_type(f, CObjectType::union(tag.clone()));
    let place = leaf
        .f
        .values()
        .member(pointee(&leaf.f, &leaf.temporary), scalar)
        .unwrap();
    let construction = vec![leaf.f.ast().assign(place, leaf.f.int(7)).unwrap()];
    let actions = leaf.drop(&leaf.first);
    let body = leaf.body_with(8, construction, actions);
    assert_eq!(
        leaf.f
            .registry
            .check_storage_paths(&[aggregate_source(&leaf.f, aggregate, body)]),
        Err(CSafetyError::UnprovedOwnership)
    );
}

#[test]
fn changing_a_live_union_arm_cannot_make_an_incomplete_owner_transferable() {
    for partial in [false, true] {
        let mut f = Fixture::new(&[]);
        let aggregate =
            CAggregateRef::Union(f.registry.declare_union(&f.file, key("Payload")).unwrap());
        let int = CObjectType::scalar(CScalarType::Int);
        let scalar = f
            .registry
            .register_member(&aggregate, key("number"), int.clone())
            .unwrap();
        let array = CObjectType::array(int, CArrayLength::new(2).unwrap()).unwrap();
        let array = f
            .registry
            .register_member(&aggregate, key("array"), array)
            .unwrap();
        f.registry
            .define_aggregate(&aggregate, vec![scalar.clone(), array.clone()])
            .unwrap();
        let CAggregateRef::Union(tag) = &aggregate else {
            unreachable!()
        };
        let mut leaf = Leaf::with_type(f, CObjectType::union(tag.clone()));
        let initial = leaf
            .f
            .values()
            .member(pointee(&leaf.f, &leaf.temporary), scalar.clone())
            .unwrap();
        let construction = vec![leaf.f.ast().assign(initial, leaf.f.int(7)).unwrap()];
        let mutation = if partial {
            let array = leaf
                .f
                .values()
                .member(pointee(&leaf.f, &leaf.first), array)
                .unwrap();
            leaf.f
                .values()
                .index(CIndexBase::Array(Box::new(array)), leaf.f.size(0))
                .unwrap()
        } else {
            leaf.f
                .values()
                .member(pointee(&leaf.f, &leaf.first), scalar)
                .unwrap()
        };
        let mut actions = vec![leaf.f.ast().assign(mutation, leaf.f.int(7)).unwrap()];
        actions.extend(leaf.moved(&leaf.first, &leaf.second));
        actions.extend(leaf.drop(&leaf.second));
        let body = leaf.body_with(8, construction, actions);
        let source = aggregate_source(&leaf.f, aggregate, body);
        assert_eq!(
            leaf.f.registry.check_storage_paths(&[source]),
            if partial {
                Err(CSafetyError::UninitializedStorage)
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn an_initialized_interior_element_is_not_a_whole_allocation_owner() {
    for interior in [false, true] {
        let mut f = Fixture::new(&[]);
        let scalar = CObjectType::scalar(CScalarType::Int);
        let array = CObjectType::array(scalar.clone(), CArrayLength::new(2).unwrap()).unwrap();
        let raw = raw(&mut f, "raw");
        let temporary = local(&mut f, pointer_type(array.clone()), "temporary");
        let owner = local(
            &mut f,
            pointer_type(if interior { scalar } else { array.clone() }),
            "owner",
        );
        f.registry.register_local_owner(&owner).unwrap();
        let allocation = descriptor(&mut f, "array", array);
        let guard = require_live(&mut f, &raw, vec![]);
        let element = |index| {
            f.values()
                .index(
                    CIndexBase::Array(Box::new(pointee(&f, &temporary))),
                    f.size(index),
                )
                .unwrap()
        };
        let selected = if interior {
            address(&f, element(0))
        } else {
            f.read(&temporary)
        };
        let body = vec![
            f.declare(&raw, allocate(&f, f.size(8))),
            guard,
            f.declare(&temporary, restore(&f, &allocation, f.read(&raw))),
            f.declare(&owner, null(&f, &owner)),
            f.ast().assign(element(0), f.int(7)).unwrap(),
            f.ast().assign(element(1), f.int(7)).unwrap(),
            assign(&f, &owner, selected),
            release(&f, erase(&f, f.read(&owner))),
            assign(&f, &owner, null(&f, &owner)),
        ];
        check(
            &f,
            body,
            if interior {
                Err(CSafetyError::UnprovedOwnership)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn complete_automatic_storage_is_not_an_owned_allocation() {
    let mut f = Fixture::new(&[]);
    let value = f.local(CScalarType::Int, "value");
    let owner = local(&mut f, pointer_type(value.ty().clone()), "owner");
    f.registry.register_local_owner(&owner).unwrap();
    let body = vec![
        f.declare(&value, f.int(7)),
        f.declare(&owner, null(&f, &owner)),
        assign(&f, &owner, address(&f, f.values().local(value).unwrap())),
    ];
    check(&f, body, Err(CSafetyError::UnprovedOwnership));
}
