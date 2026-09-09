//! Nested aggregate completeness and union activity survive dynamic selection.
use super::{
    buffer_fixture::Buffer, buffer_symbolic_fixture::within, contextual_reconstruction::key,
    numeric_fixture::Fixture, storage_fixture::*, *,
};

fn aggregate(f: &mut Fixture, union: bool) -> (CAggregateRef, CObjectType, [CMemberRef; 2]) {
    let owner = if union {
        CAggregateRef::Union(f.registry.declare_union(&f.file, key("Element")).unwrap())
    } else {
        CAggregateRef::Struct(f.registry.declare_struct(&f.file, key("Element")).unwrap())
    };
    let ty = match &owner {
        CAggregateRef::Struct(tag) => CObjectType::structure(tag.clone()),
        CAggregateRef::Union(tag) => CObjectType::union(tag.clone()),
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
fn selected_record_and_union_reads_require_the_actual_initialized_member() {
    for union in [false, true] {
        for selected in 0..2 {
            let mut f = Fixture::new(&[CScalarType::Size]);
            let (owner, ty, members) = aggregate(&mut f, union);
            let buffer = Buffer::new(&mut f, ty);
            let count = f.size(2);
            let mut body = buffer.prefix(&mut f, count);
            let field = |i: usize| {
                f.values()
                    .member(buffer.place(&f, f.input(0)), members[i].clone())
                    .unwrap()
            };
            let actions = vec![
                f.ast().assign(field(0), f.int(7)).unwrap(),
                f.discard(f.values().read(field(selected)).unwrap()),
            ];
            let index = f.input(0);
            body.push(within(&mut f, &buffer, index, actions));
            body.push(buffer.release(&f));
            let expected = if selected == 0 {
                Ok(())
            } else if union {
                Err(CSafetyError::InactiveUnionMember)
            } else {
                Err(CSafetyError::UninitializedStorage)
            };
            assert_eq!(
                f.registry
                    .check_storage_paths(&[aggregate_source(&f, owner, body)]),
                expected
            );
        }
    }
}

#[test]
fn selected_aggregate_copy_requires_a_complete_record_or_active_union_payload() {
    for union in [false, true] {
        for complete in [false, true] {
            let mut f = Fixture::new(&[CScalarType::Size]);
            let (owner, ty, members) = aggregate(&mut f, union);
            let copy = local(&mut f, ty.clone(), "copy");
            let buffer = Buffer::new(&mut f, ty);
            let count = f.size(2);
            let mut body = buffer.prefix(&mut f, count);
            body.push(f.ast().declare(copy.clone(), None).unwrap());
            let mut actions = vec![
                f.ast()
                    .assign(
                        f.values()
                            .member(buffer.place(&f, f.input(0)), members[0].clone())
                            .unwrap(),
                        f.int(7),
                    )
                    .unwrap(),
            ];
            if complete {
                actions.push(
                    f.ast()
                        .assign(
                            f.values()
                                .member(buffer.place(&f, f.input(0)), members[1].clone())
                                .unwrap(),
                            f.int(9),
                        )
                        .unwrap(),
                );
            }
            actions.push(
                f.ast()
                    .assign(
                        f.values().local(copy.clone()).unwrap(),
                        f.values().read(buffer.place(&f, f.input(0))).unwrap(),
                    )
                    .unwrap(),
            );
            actions.push(
                f.discard(
                    f.values()
                        .read(
                            f.values()
                                .member(
                                    f.values().local(copy).unwrap(),
                                    members[usize::from(complete)].clone(),
                                )
                                .unwrap(),
                        )
                        .unwrap(),
                ),
            );
            let index = f.input(0);
            body.push(within(&mut f, &buffer, index, actions));
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
fn a_possible_symbolic_alias_cannot_preserve_an_old_union_member() {
    let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size]);
    let (owner, ty, members) = aggregate(&mut f, true);
    let buffer = Buffer::new(&mut f, ty);
    let count = f.size(2);
    let mut body = buffer.prefix(&mut f, count);
    let condition = f.boolean(f.binary(
        CBinaryOperator::LogicalAnd,
        f.compare(CBinaryOperator::Less, f.input(0), f.size(2)),
        f.compare(CBinaryOperator::Less, f.input(1), f.size(2)),
    ));
    let field = |index, member: usize| {
        f.values()
            .member(buffer.place(&f, f.input(index)), members[member].clone())
            .unwrap()
    };
    let actions = vec![
        f.ast().assign(field(0, 0), f.int(7)).unwrap(),
        f.ast().assign(field(1, 1), f.int(9)).unwrap(),
        f.discard(f.values().read(field(0, 0)).unwrap()),
    ];
    body.push(f.branch(
        condition,
        vec![],
        vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
    ));
    body.extend(actions);
    body.push(buffer.release(&f));
    assert_eq!(
        f.registry
            .check_storage_paths(&[aggregate_source(&f, owner, body)]),
        Err(CSafetyError::InactiveUnionMember)
    );
}
