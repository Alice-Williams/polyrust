//! Loop phase bounds and complete initialized prefixes are separate obligations.
use super::{buffer_symbolic_fixture::*, contextual_reconstruction::key, storage_fixture::*, *};

#[test]
fn counted_buffer_access_obeys_the_actual_pre_step_phase() {
    for after_step in [false, true] {
        let (mut f, buffer, mut body) = setup();
        let counter = f.local(CScalarType::Size, "counter");
        let scope = f
            .registry
            .register_scope(&f.function, Some(&f.scope), key("iteration_body"))
            .unwrap();
        let identity = f
            .registry
            .register_loop(&f.scope, key("iteration"))
            .unwrap();
        body.push(f.declare(&counter, f.size(0)));
        let step = f
            .ast()
            .assign(
                f.values().local(counter.clone()).unwrap(),
                f.binary(CBinaryOperator::Add, f.read(&counter), f.size(1)),
            )
            .unwrap();
        let mut actions = vec![
            write(&f, &buffer, f.read(&counter), f.size(7)),
            f.discard(selected(&f, &buffer, f.read(&counter))),
        ];
        if after_step {
            actions.insert(0, step);
        } else {
            actions.push(step);
        }
        body.push(
            f.ast()
                .counted_loop(
                    identity,
                    counter.clone(),
                    buffer.count.local().clone(),
                    f.compare(
                        CBinaryOperator::Less,
                        f.read(&counter),
                        f.read(buffer.count.local()),
                    ),
                    f.ast().block(scope, actions).unwrap(),
                )
                .unwrap(),
        );
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if after_step {
                // The invalid post-step extent poisons the loop fixed point;
                // strict replay encounters its earlier uninitialized header.
                Err(CSafetyError::UninitializedStorage)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn completing_all_current_element_writes_certifies_a_buffer_prefix() {
    let (mut f, buffer, mut body) = setup();
    let counter = f.local(CScalarType::Size, "counter");
    let scope = f
        .registry
        .register_scope(&f.function, Some(&f.scope), key("iteration_body"))
        .unwrap();
    let identity = f
        .registry
        .register_loop(&f.scope, key("iteration"))
        .unwrap();
    body.push(f.declare(&counter, f.size(0)));
    let actions = vec![
        write(&f, &buffer, f.read(&counter), f.size(7)),
        f.ast()
            .assign(
                f.values().local(counter.clone()).unwrap(),
                f.binary(CBinaryOperator::Add, f.read(&counter), f.size(1)),
            )
            .unwrap(),
    ];
    body.push(
        f.ast()
            .counted_loop(
                identity,
                counter.clone(),
                buffer.count.local().clone(),
                f.compare(
                    CBinaryOperator::Less,
                    f.read(&counter),
                    f.read(buffer.count.local()),
                ),
                f.ast().block(scope, actions).unwrap(),
            )
            .unwrap(),
    );
    body.push(f.discard(buffer.read(&f, 0)));
    body.push(buffer.release(&f));
    check(&f, body, Ok(()));
}
