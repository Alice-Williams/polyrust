//! Counted-loop phases and widening also govern values reached through memory.
use super::{
    contextual_reconstruction::key, counted_fixture::Fixture, storage_fixture::pointer_type, *,
};
use crate::dialect::CKnownCall;

#[test]
fn loop_allocations_use_current_heap_writes_and_join_original_byte_ranges() {
    for lossy in [false, true] {
        let mut f = Fixture::new();
        let object = CObjectType::scalar(CScalarType::Size);
        let raw_type = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
        let raw = f
            .registry
            .register_local(&f.scope, key("raw"), raw_type.clone())
            .unwrap();
        let typed = f
            .registry
            .register_local(&f.scope, key("typed"), pointer_type(object.clone()))
            .unwrap();
        let next = f
            .registry
            .register_local(&f.body, key("next"), raw_type)
            .unwrap();
        let allocation = f
            .registry
            .register_allocation(&f.scope, key("object"), object, CAllocatorSource::Default)
            .unwrap();
        let yes = f.child(&f.scope.clone(), "yes");
        let no = f.child(&f.scope.clone(), "no");
        let e = f.expressions();
        let s = f.statements();
        let selected = e.dereference(f.read(&typed)).unwrap();
        let value = e
            .binary(
                CBinaryOperator::Add,
                f.read(&f.counter),
                f.size(if lossy { u64::MAX } else { 1 }),
            )
            .unwrap();
        let release = |value| {
            s.evaluate(
                e.call_effect(e.known(CKnownCall::Release), vec![value])
                    .unwrap(),
            )
            .unwrap()
        };
        let iteration = vec![
            s.assign(selected.clone(), value).unwrap(),
            f.declare(
                &next,
                e.call_value(
                    e.known(CKnownCall::Allocate),
                    vec![e.read(selected).unwrap()],
                )
                .unwrap(),
            ),
            release(f.read(&next)),
            f.step(),
        ];
        let nonnull = e
            .numeric_conversion(
                CScalarType::Bool,
                e.pointer_test(CPointerTest::IsNonNull(Box::new(f.read(&raw))))
                    .unwrap(),
            )
            .unwrap();
        let source = f.source(vec![
            f.declare(
                &raw,
                e.call_value(e.known(CKnownCall::Allocate), vec![f.size(8)])
                    .unwrap(),
            ),
            s.if_statement(
                nonnull,
                s.block(yes, vec![]).unwrap(),
                s.block(no, vec![s.return_statement(None).unwrap()])
                    .unwrap(),
            )
            .unwrap(),
            f.declare(
                &typed,
                e.allocation_restore(allocation, f.read(&raw)).unwrap(),
            ),
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(2)),
            f.iteration(iteration),
            release(f.read(&raw)),
        ]);
        let result = f.registry.check_storage_paths(&[source]);
        if lossy {
            // A failed allocation poisons provisional loop state. Strict
            // replay may reject that state before reaching the original loss.
            assert!(
                matches!(
                    result,
                    Err(CSafetyError::UnprovedSizeArithmetic | CSafetyError::UninitializedStorage)
                ),
                "{result:?}"
            );
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}
