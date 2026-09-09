//! Each fresh call in a repeated scope captures a fresh count and empty storage.
use super::{
    buffer_fixture::Buffer, contextual_reconstruction::key, numeric_fixture::Fixture,
    storage_fixture::*, *,
};

#[test]
fn repeated_count_scope_allocation_captures_each_new_count_activation() {
    for initialized in [false, true] {
        let mut f = Fixture::new(&[]);
        let counter = f.local(CScalarType::Size, "counter");
        let bound = f
            .registry
            .register_local(
                &f.scope,
                key("bound"),
                CObjectType::scalar(CScalarType::Size)
                    .with_constness(CConstness::Const)
                    .unwrap(),
            )
            .unwrap();
        let identity = f
            .registry
            .register_loop(&f.scope, key("iteration"))
            .unwrap();
        let child = f
            .registry
            .register_scope(&f.function, Some(&f.scope), key("iteration_body"))
            .unwrap();
        let parent = f.scope.clone();
        f.scope = child.clone();
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Size));
        let count = f.binary(CBinaryOperator::Add, f.read(&counter), f.size(2));
        let mut actions = buffer.prefix(&mut f, count);
        if initialized {
            actions.push(buffer.write(&f, 0, f.size(7)));
        }
        actions.push(f.discard(buffer.read(&f, 0)));
        actions.push(buffer.release(&f));
        actions.push(
            f.ast()
                .assign(
                    f.values().local(counter.clone()).unwrap(),
                    f.binary(CBinaryOperator::Add, f.read(&counter), f.size(1)),
                )
                .unwrap(),
        );
        f.scope = parent;
        let iteration = f
            .ast()
            .counted_loop(
                identity,
                counter.clone(),
                bound.clone(),
                f.compare(CBinaryOperator::Less, f.read(&counter), f.read(&bound)),
                f.ast().block(child, actions).unwrap(),
            )
            .unwrap();
        let body = vec![
            f.declare(&counter, f.size(0)),
            f.declare(&bound, f.size(3)),
            iteration,
        ];
        check(
            &f,
            body,
            if initialized {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}
