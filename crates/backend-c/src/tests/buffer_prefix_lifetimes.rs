//! Full coverage does not outlive allocation storage or dependent addresses.
use super::{
    allocation_fixture::*, buffer_fixture::Buffer, buffer_prefix_fixture::Counted,
    buffer_symbolic_fixture::*, contextual_reconstruction::key, heap_fixture::*,
    numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn release_expires_even_a_previously_complete_prefix() {
    let (mut f, buffer, mut body) = setup();
    let construction = Counted::new(&mut f, "construct");
    body.push(construction.declaration(&f));
    body.push(construction.finish(
        &f,
        buffer.count.local(),
        vec![
            write(&f, &buffer, f.read(&construction.counter), f.size(7)),
            construction.step(&f),
        ],
    ));
    body.push(buffer.release(&f));
    body.push(f.discard(buffer.read(&f, 0)));
    assert_eq!(
        f.registry.check_storage_paths(&[f.source(body)]),
        Err(CSafetyError::ExpiredStorage)
    );
}

#[test]
fn a_complete_original_prefix_survives_count_scope_exit_for_constant_access() {
    let mut f = Fixture::new(&[]);
    let child = f
        .registry
        .register_scope(&f.function, Some(&f.scope), key("construction_scope"))
        .unwrap();
    let element = CObjectType::scalar(CScalarType::Size);
    let buffer = Buffer::scoped(&mut f, &child, element.clone());
    let mut body = vec![
        f.declare(&buffer.raw, null(&f, &buffer.raw)),
        f.declare(&buffer.data, null(&f, &buffer.data)),
    ];
    let parent = std::mem::replace(&mut f.scope, child.clone());
    let construction = Counted::new(&mut f, "construct");
    let live = require_live(&mut f, &buffer.raw, vec![]);
    let mut inner = vec![
        f.declare(buffer.count.local(), f.size(2)),
        assign(
            &f,
            &buffer.raw,
            allocate(
                &f,
                f.binary(
                    CBinaryOperator::Multiply,
                    f.read(buffer.count.local()),
                    f.values().size_of(element).unwrap(),
                ),
            ),
        ),
        live,
        assign(
            &f,
            &buffer.data,
            restore(&f, &buffer.descriptor, f.read(&buffer.raw)),
        ),
        construction.declaration(&f),
    ];
    inner.push(construction.finish(
        &f,
        buffer.count.local(),
        vec![
            write(&f, &buffer, f.read(&construction.counter), f.size(7)),
            construction.step(&f),
        ],
    ));
    f.scope = parent;
    body.push(
        f.ast()
            .nested_block(f.ast().block(child, inner).unwrap())
            .unwrap(),
    );
    body.push(f.discard(buffer.read(&f, 1)));
    body.push(buffer.release(&f));
    assert_eq!(f.registry.check_storage_paths(&[f.source(body)]), Ok(()));
}

#[test]
fn a_saved_prefix_address_is_retired_when_its_index_changes() {
    for mutate in [false, true] {
        let (mut f, buffer, mut body) = setup();
        let construction = Counted::new(&mut f, "construct");
        let saved = local(
            &mut f,
            pointer_type(CObjectType::scalar(CScalarType::Size)),
            "saved",
        );
        body.push(construction.declaration(&f));
        body.push(construction.finish(
            &f,
            buffer.count.local(),
            vec![
                write(&f, &buffer, f.read(&construction.counter), f.size(7)),
                construction.step(&f),
            ],
        ));
        let guard = f.compare(
            CBinaryOperator::Less,
            f.input(1),
            f.read(buffer.count.local()),
        );
        body.push(f.branch(
            guard,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        body.push(f.declare(&saved, address(&f, buffer.place(&f, f.input(1)))));
        if mutate {
            body.push(
                f.ast()
                    .assign(
                        f.values().parameter(f.parameters[1].clone()).unwrap(),
                        f.size(0),
                    )
                    .unwrap(),
            );
        }
        body.push(f.discard(pointed_read(&f, f.read(&saved))));
        body.push(buffer.release(&f));
        assert_eq!(
            f.registry.check_storage_paths(&[f.source(body)]),
            if mutate {
                Err(CSafetyError::UnprovedStorage)
            } else {
                Ok(())
            }
        );
    }
}
