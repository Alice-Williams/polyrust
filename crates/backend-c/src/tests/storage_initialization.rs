//! Exact partial initialization and union payload facts survive only valid paths.
use super::{
    contextual_reconstruction::key, index_extent_fixture as arrays, numeric_fixture::Fixture,
    storage_fixture::*, *,
};

#[test]
fn writing_one_field_does_not_initialize_a_same_typed_neighbor() {
    for selected in 0..2 {
        let mut f = Fixture::new(&[]);
        let tag = f.registry.declare_struct(&f.file, key("Record")).unwrap();
        let owner = CAggregateRef::Struct(tag.clone());
        let members = ["first", "second"].map(|name| {
            f.registry
                .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Int))
                .unwrap()
        });
        f.registry
            .define_aggregate(&owner, members.to_vec())
            .unwrap();
        let value = local(&mut f, CObjectType::structure(tag), "record");
        let field = |i: usize| {
            f.values()
                .member(f.values().local(value.clone()).unwrap(), members[i].clone())
                .unwrap()
        };
        let source = aggregate_source(
            &f,
            owner,
            vec![
                f.ast().declare(value.clone(), None).unwrap(),
                f.ast().assign(field(0), f.int(1)).unwrap(),
                f.discard(pointed_read(&f, address(&f, field(selected)))),
            ],
        );
        f.registry
            .check_numeric_flow(std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            if selected == 0 {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            }
        );
    }
}

#[test]
fn partial_array_prefix_is_not_complete_storage() {
    for prefix in 0..=2 {
        for selected in 0..2 {
            let mut f = Fixture::new(&[]);
            let array = arrays::array(&mut f, 2, "array");
            let base = address(&f, arrays::place(&f, &array, f.size(0)));
            let mut body = vec![f.ast().declare(array, None).unwrap()];
            for index in 0..prefix {
                body.push(
                    f.ast()
                        .assign(pointer_index(&f, base.clone(), index), f.int(7))
                        .unwrap(),
                );
            }
            body.push(f.discard(f.values().read(pointer_index(&f, base, selected)).unwrap()));
            check(
                &f,
                body,
                if selected < prefix {
                    Ok(())
                } else {
                    Err(CSafetyError::UninitializedStorage)
                },
            );
        }
    }
}

#[test]
fn union_zero_and_writes_establish_only_the_actual_active_member() {
    for zero in [false, true] {
        for read_member in 0..2 {
            let mut f = Fixture::new(&[]);
            let tag = f.registry.declare_union(&f.file, key("Payload")).unwrap();
            let owner = CAggregateRef::Union(tag.clone());
            let members = ["first", "second"].map(|name| {
                f.registry
                    .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Int))
                    .unwrap()
            });
            f.registry
                .define_aggregate(&owner, members.to_vec())
                .unwrap();
            let value = local(&mut f, CObjectType::union(tag), "payload");
            let selected = |index: usize| {
                f.values()
                    .member(
                        f.values().local(value.clone()).unwrap(),
                        members[index].clone(),
                    )
                    .unwrap()
            };
            let mut body = vec![arrays::declare(&f, &value)];
            if !zero {
                body.push(f.ast().assign(selected(1), f.int(8)).unwrap());
            }
            body.push(f.discard(f.values().read(selected(read_member)).unwrap()));
            let source = aggregate_source(&f, owner, body);
            assert_eq!(
                f.registry.check_storage_paths(&[source]),
                if read_member == usize::from(!zero) {
                    Ok(())
                } else {
                    Err(CSafetyError::InactiveUnionMember)
                }
            );
        }
    }
}

#[test]
fn member_pointer_does_not_reach_a_neighbor_field() {
    for offset in [0, 1] {
        let mut f = Fixture::new(&[]);
        let tag = f.registry.declare_struct(&f.file, key("Record")).unwrap();
        let owner = CAggregateRef::Struct(tag.clone());
        let members = ["first", "second"].map(|name| {
            f.registry
                .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Int))
                .unwrap()
        });
        f.registry
            .define_aggregate(&owner, members.to_vec())
            .unwrap();
        let value = local(&mut f, CObjectType::structure(tag), "record");
        let member = f
            .values()
            .member(f.values().local(value.clone()).unwrap(), members[0].clone())
            .unwrap();
        let pointer = address(&f, member);
        let source = aggregate_source(
            &f,
            owner,
            vec![
                arrays::declare(&f, &value),
                f.discard(f.values().read(pointer_index(&f, pointer, offset)).unwrap()),
            ],
        );
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            if offset == 0 {
                Ok(())
            } else {
                Err(CSafetyError::IndexOutOfBounds)
            }
        );
    }
}
