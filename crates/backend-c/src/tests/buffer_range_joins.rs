//! One unchanged selected element keeps its identity across guard refinements.
use super::{
    allocation_fixture::*, buffer_fixture::Buffer, buffer_symbolic_fixture::*,
    heap_fixture::assign, numeric_fixture::Fixture, storage_fixture::*, *,
};

fn guarded() -> (Fixture, Buffer, Vec<CStatement>) {
    let (mut f, buffer, mut body) = setup();
    let condition = f.compare(
        CBinaryOperator::Less,
        f.input(1),
        f.read(buffer.count.local()),
    );
    body.push(f.branch(
        condition,
        vec![],
        vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
    ));
    (f, buffer, body)
}

#[test]
fn differently_refined_branches_join_numeric_values_and_keep_wrapped_history() {
    for wrapped in [false, true] {
        let (mut f, buffer, mut body) = guarded();
        let extra = raw(&mut f, "extra");
        let condition = f.compare(CBinaryOperator::Less, f.input(1), f.size(1));
        let value = if wrapped {
            f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(8))
        } else {
            f.size(7)
        };
        let yes = vec![write(&f, &buffer, f.input(1), f.size(7))];
        let no = vec![write(&f, &buffer, f.input(1), value)];
        body.push(f.branch(condition, yes, no));
        body.push(f.declare(&extra, allocate(&f, selected(&f, &buffer, f.input(1)))));
        body.push(release(&f, f.read(&extra)));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if wrapped {
                Err(CSafetyError::UnprovedSizeArithmetic)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn differently_refined_branches_preserve_saved_addresses_until_index_mutation() {
    for mutation in [false, true] {
        for write_after in [false, true] {
            let (mut f, buffer, mut body) = guarded();
            let saved = local(
                &mut f,
                pointer_type(CObjectType::scalar(CScalarType::Size)),
                "saved",
            );
            body.push(f.declare(&saved, null(&f, &saved)));
            let condition = f.compare(CBinaryOperator::Less, f.input(1), f.size(1));
            let mut actions = vec![];
            if !write_after {
                actions.push(write(&f, &buffer, f.input(1), f.size(7)));
            }
            actions.push(assign(
                &f,
                &saved,
                address(&f, buffer.place(&f, f.input(1))),
            ));
            body.push(f.branch(condition, actions.clone(), actions));
            if mutation {
                body.push(
                    f.ast()
                        .assign(
                            f.values().parameter(f.parameters[1].clone()).unwrap(),
                            f.size(0),
                        )
                        .unwrap(),
                );
            }
            if write_after {
                body.push(
                    f.ast()
                        .assign(f.values().dereference(f.read(&saved)).unwrap(), f.size(7))
                        .unwrap(),
                );
                body.push(f.discard(selected(&f, &buffer, f.input(1))));
            } else {
                body.push(f.discard(pointed_read(&f, f.read(&saved))));
            }
            body.push(buffer.release(&f));
            check(
                &f,
                body,
                if mutation {
                    Err(CSafetyError::UnprovedStorage)
                } else {
                    Ok(())
                },
            );
        }
    }
}

#[test]
fn narrowing_an_unchanged_index_preserves_the_reaching_numeric_write() {
    for wrapped in [false, true] {
        let (mut f, buffer, mut body) = guarded();
        let extra = raw(&mut f, "extra");
        let value = if wrapped {
            f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(8))
        } else {
            f.size(7)
        };
        body.push(write(&f, &buffer, f.input(1), value));
        let condition = f.compare(CBinaryOperator::Less, f.input(1), f.size(1));
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        body.push(f.declare(&extra, allocate(&f, selected(&f, &buffer, f.input(1)))));
        body.push(release(&f, f.read(&extra)));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if wrapped {
                Err(CSafetyError::UnprovedSizeArithmetic)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn different_index_bindings_do_not_become_one_saved_target_at_a_join() {
    let (mut f, buffer, mut body) = guarded();
    let saved = local(
        &mut f,
        pointer_type(CObjectType::scalar(CScalarType::Size)),
        "saved",
    );
    body.push(f.declare(&saved, null(&f, &saved)));
    let condition = f.compare(
        CBinaryOperator::Less,
        f.input(2),
        f.read(buffer.count.local()),
    );
    body.push(f.branch(
        condition,
        vec![],
        vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
    ));
    let condition = f.compare(CBinaryOperator::Less, f.input(1), f.size(1));
    let yes = vec![
        write(&f, &buffer, f.input(1), f.size(7)),
        assign(&f, &saved, address(&f, buffer.place(&f, f.input(1)))),
    ];
    let no = vec![
        write(&f, &buffer, f.input(2), f.size(7)),
        assign(&f, &saved, address(&f, buffer.place(&f, f.input(2)))),
    ];
    body.push(f.branch(condition, yes, no));
    body.push(f.discard(pointed_read(&f, f.read(&saved))));
    body.push(buffer.release(&f));
    check(&f, body, Err(CSafetyError::UnprovedStorage));
}

#[test]
fn singleton_saved_target_joins_retain_binding_dependencies() {
    for other in 0..3 {
        for mutation in [false, true] {
            let (mut f, buffer, mut body) = guarded();
            for input in [1, 2] {
                let condition = f.compare(CBinaryOperator::Less, f.input(input), f.size(1));
                body.push(f.branch(
                    condition,
                    vec![],
                    vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
                ));
            }
            let saved = local(
                &mut f,
                pointer_type(CObjectType::scalar(CScalarType::Size)),
                "saved",
            );
            body.push(f.declare(&saved, null(&f, &saved)));
            let right = match other {
                0 => f.input(2),
                1 => f.size(0),
                _ => f.input(1),
            };
            let actions = |index: CValue| {
                vec![
                    write(&f, &buffer, index.clone(), f.size(7)),
                    assign(&f, &saved, address(&f, buffer.place(&f, index))),
                ]
            };
            let yes = actions(f.input(1));
            let no = actions(right);
            let condition = f.compare(CBinaryOperator::Less, f.input(0), f.size(3));
            body.push(f.branch(condition, yes, no));
            if mutation {
                body.push(
                    f.ast()
                        .assign(
                            f.values().parameter(f.parameters[1].clone()).unwrap(),
                            f.size(1),
                        )
                        .unwrap(),
                );
            }
            body.push(f.discard(pointed_read(&f, f.read(&saved))));
            body.push(buffer.release(&f));
            check(
                &f,
                body,
                if other == 2 && !mutation {
                    Ok(())
                } else {
                    Err(CSafetyError::UnprovedStorage)
                },
            );
        }
    }
}

#[test]
fn duplicate_singleton_observations_do_not_cover_an_unwritten_offset() {
    for complete in [false, true] {
        let (mut f, buffer, mut body) = guarded();
        let left = f.local(CScalarType::Size, "left");
        let right = f.local(CScalarType::Size, "right");
        body.push(f.declare(&left, f.size(0)));
        body.push(f.declare(&right, f.size(0)));
        let condition = f.compare(CBinaryOperator::Less, f.input(1), f.size(2));
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        body.push(write(&f, &buffer, f.read(&left), f.size(7)));
        body.push(write(&f, &buffer, f.read(&right), f.size(7)));
        if complete {
            body.push(write(&f, &buffer, f.size(1), f.size(7)));
        }
        body.push(f.discard(selected(&f, &buffer, f.input(1))));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if complete {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}
