//! A later allocation activation cannot inherit an earlier complete buffer.
use super::{
    buffer_fixture::Buffer,
    buffer_prefix_fixture::{Counted, immutable_size},
    numeric_fixture::Fixture,
    *,
};

#[test]
fn repeated_allocations_require_complete_construction_again_on_every_activation() {
    for every_time in [false, true] {
        let mut f = Fixture::new(&[]);
        let outer = Counted::new(&mut f, "outer");
        let outer_bound = immutable_size(&mut f, "outer_count");
        let parent = std::mem::replace(&mut f.scope, outer.scope.clone());
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Size));
        let construction = Counted::new(&mut f, "construct");
        let count = f.binary(CBinaryOperator::Add, f.read(&outer.counter), f.size(2));
        let mut actions = buffer.prefix(&mut f, count);
        actions.push(construction.declaration(&f));
        let write = f
            .ast()
            .assign(buffer.place(&f, f.read(&construction.counter)), f.size(7))
            .unwrap();
        let mut writes = if every_time {
            vec![write]
        } else {
            let condition = f.compare(CBinaryOperator::Equal, f.read(&outer.counter), f.size(0));
            vec![construction.branch(&mut f, condition, vec![write], vec![])]
        };
        writes.push(construction.step(&f));
        actions.push(construction.finish(&f, buffer.count.local(), writes));
        actions.push(f.discard(buffer.read(&f, 0)));
        actions.push(buffer.release(&f));
        actions.push(outer.step(&f));
        f.scope = parent;
        let body = vec![
            outer.declaration(&f),
            f.declare(&outer_bound, f.size(3)),
            outer.finish(&f, &outer_bound, actions),
        ];
        let result = f.registry.check_storage_paths(&[f.source(body)]);
        if every_time {
            assert_eq!(result, Ok(()));
        } else {
            assert!(result.is_err(), "{result:?}");
        }
    }
}
