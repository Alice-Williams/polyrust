//! Loop bounds are proved algebraically, including SizeMax, without unrolling.
use super::counted_fixture::Fixture;
use super::*;
use crate::dialect::CKnownCall;

#[test]
fn numeric_loop_proof_handles_zero_one_many_and_size_max_bounds() {
    for bound in [0, 1, 7, u64::MAX] {
        let f = Fixture::new();
        let source = f.source(vec![
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(bound)),
            f.iteration(vec![f.step()]),
        ]);
        f.registry.check_numeric_flow(&[source]).unwrap();
    }
}

#[test]
fn zero_iteration_body_is_not_numerically_evaluated() {
    let f = Fixture::new();
    let bad = f
        .statements()
        .discard(
            f.expressions()
                .binary(CBinaryOperator::Divide, f.size(1), f.size(0))
                .unwrap(),
        )
        .unwrap();
    for bound in [0, 1] {
        let source = f.source(vec![
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(bound)),
            f.iteration(vec![bad.clone(), f.step()]),
        ]);
        let result = f.registry.check_numeric_flow(&[source]);
        if bound == 0 {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CSafetyError::DivisionByZero));
        }
    }
}

#[test]
fn after_step_counter_cannot_reuse_the_strict_before_step_range() {
    for after in [false, true] {
        let f = Fixture::new();
        let values = f.expressions();
        let bytes = values
            .binary(CBinaryOperator::Add, f.read(&f.counter), f.size(1))
            .unwrap();
        let allocate = f
            .statements()
            .discard(
                values
                    .call_value(values.known(CKnownCall::Allocate), vec![bytes])
                    .unwrap(),
            )
            .unwrap();
        let body = if after {
            vec![f.step(), allocate]
        } else {
            vec![allocate, f.step()]
        };
        let source = f.source(vec![
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(u64::MAX)),
            f.iteration(body),
        ]);
        let result = f.registry.check_numeric_flow(&[source]);
        if after {
            assert_eq!(result, Err(CSafetyError::UnprovedSizeArithmetic));
        } else {
            result.unwrap();
        }
    }
}
