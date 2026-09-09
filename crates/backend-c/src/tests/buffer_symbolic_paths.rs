//! Selected-element facts require the same unchanged binding, not equal intervals.
use super::{allocation_fixture::null, heap_fixture::assign};
use super::{buffer_symbolic_fixture::*, storage_fixture::*, *};

#[test]
fn a_runtime_selected_write_initializes_that_same_selected_element() {
    let (mut f, buffer, mut body) = setup();
    let actions = vec![
        write(&f, &buffer, f.input(1), f.size(7)),
        f.discard(selected(&f, &buffer, f.input(1))),
    ];
    let index = f.input(1);
    body.push(within(&mut f, &buffer, index, actions));
    body.push(buffer.release(&f));
    check(&f, body, Ok(()));
}

#[test]
fn another_in_range_index_does_not_inherit_a_selected_write() {
    let (mut f, buffer, mut body) = setup();
    let first = f.compare(
        CBinaryOperator::Less,
        f.input(1),
        f.read(buffer.count.local()),
    );
    let second = f.compare(
        CBinaryOperator::Less,
        f.input(2),
        f.read(buffer.count.local()),
    );
    let condition = f.boolean(f.binary(CBinaryOperator::LogicalAnd, first, second));
    let actions = vec![
        write(&f, &buffer, f.input(1), f.size(7)),
        f.discard(selected(&f, &buffer, f.input(2))),
    ];
    body.push(f.branch(condition, actions, vec![]));
    body.push(buffer.release(&f));
    check(&f, body, Err(CSafetyError::UninitializedStorage));
}

#[test]
fn index_mutation_through_an_alias_retires_selected_cells_and_saved_pointer_equivalence() {
    for saved in [false, true] {
        let (mut f, buffer, mut body) = setup();
        let index = f.local(CScalarType::Size, "index");
        let index_alias = local(
            &mut f,
            pointer_type(CObjectType::scalar(CScalarType::Size)),
            "index_alias",
        );
        let data_alias = local(
            &mut f,
            pointer_type(CObjectType::scalar(CScalarType::Size)),
            "data_alias",
        );
        body.push(f.declare(&index, f.input(1)));
        body.push(f.declare(
            &index_alias,
            address(&f, f.values().local(index.clone()).unwrap()),
        ));
        body.push(f.declare(&data_alias, null(&f, &data_alias)));
        let actions = vec![
            write(&f, &buffer, f.read(&index), f.size(9)),
            assign(
                &f,
                &data_alias,
                address(&f, buffer.place(&f, f.read(&index))),
            ),
            f.ast()
                .assign(
                    f.values().dereference(f.read(&index_alias)).unwrap(),
                    f.size(0),
                )
                .unwrap(),
            f.discard(if saved {
                pointed_read(&f, f.read(&data_alias))
            } else {
                selected(&f, &buffer, f.read(&index))
            }),
        ];
        let current = f.read(&index);
        // Keep the invalid action outside a branch join: otherwise a shorter
        // successor can report provisional poisoning before this diagnostic.
        let condition = f.compare(CBinaryOperator::Less, current, f.read(buffer.count.local()));
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        body.extend(actions);
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            Err(if saved {
                CSafetyError::UnprovedStorage
            } else {
                CSafetyError::UninitializedStorage
            }),
        );
    }
}
