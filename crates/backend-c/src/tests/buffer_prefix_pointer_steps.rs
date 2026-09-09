//! Advancing a frontier cannot restore an address invalidated by that same step.
use super::{
    allocation_fixture::*, buffer_fixture::Buffer, buffer_prefix_fixture::Counted,
    contextual_reconstruction::key, heap_fixture::*, numeric_fixture::Fixture, storage_fixture::*,
    *,
};

#[test]
fn prefix_growth_preserves_constant_addresses_but_retires_current_index_addresses() {
    for current_index in [false, true] {
        let mut f = Fixture::new(&[]);
        let source = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Int));
        let count = f.size(2);
        let mut body = source.prefix(&mut f, count);
        body.push(source.write(&f, 0, f.int(7)));
        body.push(source.write(&f, 1, f.int(9)));
        let child = f
            .registry
            .register_scope(&f.function, Some(&f.scope), key("pointers"))
            .unwrap();
        let parent = std::mem::replace(&mut f.scope, child.clone());
        let element = pointer_type(CObjectType::scalar(CScalarType::Int));
        let destination = Buffer::named(&mut f, element.clone(), "destination");
        let construction = Counted::new(&mut f, "construct");
        let failure_cleanup = vec![source.release(&f)];
        let live = require_live(&mut f, &destination.raw, failure_cleanup);
        let mut inner = vec![
            f.declare(destination.count.local(), f.size(2)),
            f.declare(
                &destination.raw,
                allocate(
                    &f,
                    f.binary(
                        CBinaryOperator::Multiply,
                        f.read(destination.count.local()),
                        f.values().size_of(element).unwrap(),
                    ),
                ),
            ),
            live,
            f.declare(
                &destination.data,
                restore(&f, &destination.descriptor, f.read(&destination.raw)),
            ),
            construction.declaration(&f),
        ];
        let index = if current_index {
            f.read(&construction.counter)
        } else {
            f.size(0)
        };
        let pointer = address(&f, source.place(&f, index));
        inner.push(construction.finish(
            &f,
            destination.count.local(),
            vec![
            f.ast().assign(destination.place(&f, f.read(&construction.counter)), pointer).unwrap(),
            construction.step(&f),
        ],
        ));
        inner.push(f.discard(pointed_read(&f, destination.read(&f, 0))));
        inner.push(destination.release(&f));
        f.scope = parent;
        body.push(
            f.ast()
                .nested_block(f.ast().block(child, inner).unwrap())
                .unwrap(),
        );
        body.push(source.release(&f));
        let result = f.registry.check_storage_paths(&[f.source(body)]);
        if current_index {
            assert!(
                result.is_err(),
                "counter update must retire the captured address: {result:?}"
            );
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}
