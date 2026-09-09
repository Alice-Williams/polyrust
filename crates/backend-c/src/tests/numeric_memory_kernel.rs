//! Memory resolution reuses the standalone transfer kernel, including failures.
use super::{numeric_fixture::Fixture, numeric_memory_fixture::*, *};

#[test]
fn direct_and_heap_numeric_transfers_have_identical_boundary_results() {
    for operation in [
        CBinaryOperator::Add,
        CBinaryOperator::Subtract,
        CBinaryOperator::Multiply,
        CBinaryOperator::Divide,
        CBinaryOperator::Remainder,
        CBinaryOperator::ShiftLeft,
        CBinaryOperator::ShiftRight,
    ] {
        for left in [0, 1, 2, u64::MAX / 2, u64::MAX] {
            for right in [0, 1, 2, 63, 64, u64::MAX] {
                let mut f = Fixture::new(&[]);
                let local = f.local(CScalarType::Size, "local");
                let expected = f.check(vec![
                    f.declare(&local, f.size(left)),
                    f.discard(f.binary(operation, f.read(&local), f.size(right))),
                ]);
                let mut f = Fixture::new(&[]);
                let heap = Heap::new(&mut f, "heap", CObjectType::scalar(CScalarType::Size));
                let mut body = heap.prefix(&mut f, 8, vec![]);
                body.extend([
                    heap.write(&f, f.size(left)),
                    f.discard(f.binary(operation, heap.read(&f), f.size(right))),
                    heap.release(&f),
                ]);
                assert_eq!(
                    f.registry.check_storage_paths(&[f.source(body)]),
                    expected,
                    "{left} {operation:?} {right}"
                );
            }
        }
    }
}
