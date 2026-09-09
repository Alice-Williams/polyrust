//! Prefix growth requires complete nested representations, not one observed field.
use super::{
    buffer_fixture::Buffer, buffer_prefix_fixture::Counted, contextual_reconstruction::key,
    numeric_fixture::Fixture, storage_fixture::aggregate_source, *,
};

fn aggregate(f: &mut Fixture, union: bool) -> (CAggregateRef, CObjectType, [CMemberRef; 2]) {
    let owner = if union {
        CAggregateRef::Union(f.registry.declare_union(&f.file, key("Element")).unwrap())
    } else {
        CAggregateRef::Struct(f.registry.declare_struct(&f.file, key("Element")).unwrap())
    };
    let ty = match &owner {
        CAggregateRef::Union(tag) => CObjectType::union(tag.clone()),
        CAggregateRef::Struct(tag) => CObjectType::structure(tag.clone()),
    };
    let members = ["first", "second"].map(|name| {
        f.registry
            .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Int))
            .unwrap()
    });
    f.registry
        .define_aggregate(&owner, members.to_vec())
        .unwrap();
    (owner, ty, members)
}

#[test]
fn aggregate_prefixes_require_every_struct_field_or_the_active_union_payload() {
    for union in [false, true] {
        for complete in [false, true] {
            let mut f = Fixture::new(&[]);
            let (owner, ty, members) = aggregate(&mut f, union);
            let buffer = Buffer::new(&mut f, ty);
            let construction = Counted::new(&mut f, "construct");
            let count = f.size(2);
            let mut body = buffer.prefix(&mut f, count);
            body.push(construction.declaration(&f));
            let mut actions = Vec::new();
            for member in members.iter().take(if complete { 2 } else { 1 }) {
                actions.push(
                    f.ast()
                        .assign(
                            f.values()
                                .member(
                                    buffer.place(&f, f.read(&construction.counter)),
                                    member.clone(),
                                )
                                .unwrap(),
                            f.int(7),
                        )
                        .unwrap(),
                );
            }
            actions.push(construction.step(&f));
            body.push(construction.finish(&f, buffer.count.local(), actions));
            body.push(f.discard(buffer.read(&f, 0)));
            body.push(buffer.release(&f));
            assert_eq!(
                f.registry
                    .check_storage_paths(&[aggregate_source(&f, owner, body)]),
                if union || complete {
                    Ok(())
                } else {
                    Err(CSafetyError::UninitializedStorage)
                }
            );
        }
    }
}

#[test]
fn nested_array_prefixes_do_not_invent_missing_inner_elements() {
    for complete in [false, true] {
        let mut f = Fixture::new(&[]);
        let array = CObjectType::array(
            CObjectType::scalar(CScalarType::Int),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let buffer = Buffer::new(&mut f, array);
        let construction = Counted::new(&mut f, "construct");
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        body.push(construction.declaration(&f));
        let mut actions = Vec::new();
        for inner in 0..if complete { 2 } else { 1 } {
            let place = f
                .values()
                .index(
                    CIndexBase::Array(Box::new(buffer.place(&f, f.read(&construction.counter)))),
                    f.size(inner),
                )
                .unwrap();
            actions.push(f.ast().assign(place, f.int(7)).unwrap());
        }
        actions.push(construction.step(&f));
        body.push(construction.finish(&f, buffer.count.local(), actions));
        let place = f
            .values()
            .index(
                CIndexBase::Array(Box::new(buffer.place(&f, f.size(0)))),
                f.size(1),
            )
            .unwrap();
        body.push(f.discard(f.values().read(place).unwrap()));
        body.push(buffer.release(&f));
        assert_eq!(
            f.registry.check_storage_paths(&[f.source(body)]),
            if complete {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            }
        );
    }
}

#[test]
fn later_possible_alias_writes_preserve_struct_fields_but_not_an_old_union_arm() {
    for union in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let (owner, ty, members) = aggregate(&mut f, union);
        let buffer = Buffer::new(&mut f, ty);
        let construction = Counted::new(&mut f, "construct");
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        body.push(construction.declaration(&f));
        let mut actions = Vec::new();
        for member in members.iter().take(if union { 1 } else { 2 }) {
            actions.push(
                f.ast()
                    .assign(
                        f.values()
                            .member(
                                buffer.place(&f, f.read(&construction.counter)),
                                member.clone(),
                            )
                            .unwrap(),
                        f.int(7),
                    )
                    .unwrap(),
            );
        }
        actions.push(construction.step(&f));
        body.push(construction.finish(&f, buffer.count.local(), actions));
        let condition = f.compare(CBinaryOperator::Less, f.input(0), f.size(2));
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        let destination = f
            .values()
            .member(buffer.place(&f, f.input(0)), members[1].clone())
            .unwrap();
        body.push(f.ast().assign(destination, f.int(9)).unwrap());
        let first = f
            .values()
            .member(buffer.place(&f, f.size(0)), members[0].clone())
            .unwrap();
        body.push(f.discard(f.values().read(first).unwrap()));
        body.push(buffer.release(&f));
        let result = f
            .registry
            .check_storage_paths(&[aggregate_source(&f, owner, body)]);
        if union {
            assert!(result.is_err(), "{result:?}");
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}
