//! Numeric consumers exercise the actual structural-phase/solver handoff.
use super::{contextual_reconstruction::key, counted_fixture::Fixture, *};
use crate::dialect::CKnownCall;

fn allocation(f: &Fixture) -> CStatement {
    let values = f.expressions();
    let bytes = values
        .binary(CBinaryOperator::Add, f.read(&f.counter), f.size(1))
        .unwrap();
    f.statements()
        .discard(
            values
                .call_value(values.known(CKnownCall::Allocate), vec![bytes])
                .unwrap(),
        )
        .unwrap()
}
fn check(f: &Fixture, body: Vec<CStatement>, accepted: bool) {
    let source = f.source(vec![
        f.declare(&f.counter, f.size(0)),
        f.declare(&f.bound, f.size(u64::MAX)),
        f.iteration(body),
    ]);
    let result = f.registry.check_numeric_flow(&[source]);
    if accepted {
        result.unwrap();
    } else {
        assert_eq!(result, Err(CSafetyError::UnprovedSizeArithmetic));
    }
}
fn parity(f: &Fixture) -> CValue {
    let values = f.expressions();
    let remainder = values
        .binary(CBinaryOperator::Remainder, f.read(&f.counter), f.size(2))
        .unwrap();
    values
        .numeric_conversion(
            CScalarType::Bool,
            values
                .binary(CBinaryOperator::Equal, remainder, f.size(0))
                .unwrap(),
        )
        .unwrap()
}

#[test]
fn branch_local_steps_and_continue_do_not_transplant_before_step_proof() {
    for after in [false, true] {
        for continuing in [false, true] {
            let mut f = Fixture::new();
            let yes = f.child(&f.body.clone(), "yes");
            let no = f.child(&f.body.clone(), "no");
            let mut left = if after {
                vec![f.step(), allocation(&f)]
            } else {
                vec![allocation(&f), f.step()]
            };
            if continuing {
                left.push(
                    f.statements()
                        .continue_statement(f.identity.clone())
                        .unwrap(),
                );
            }
            let branch = f
                .statements()
                .if_statement(
                    parity(&f),
                    f.statements().block(yes, left).unwrap(),
                    f.statements()
                        .block(no, vec![allocation(&f), f.step()])
                        .unwrap(),
                )
                .unwrap();
            check(&f, vec![branch], !after);
        }
    }
}

#[test]
fn joining_distinct_branch_updates_retains_only_after_step_facts() {
    let mut f = Fixture::new();
    let yes = f.child(&f.body.clone(), "yes");
    let no = f.child(&f.body.clone(), "no");
    let branch = f
        .statements()
        .if_statement(
            parity(&f),
            f.statements().block(yes, vec![f.step()]).unwrap(),
            f.statements().block(no, vec![f.step()]).unwrap(),
        )
        .unwrap();
    check(&f, vec![branch, allocation(&f)], false);
}

#[test]
fn nested_loop_cycles_and_continue_preserve_the_outer_phase() {
    for outer_after in [false, true] {
        for continuing in [false, true] {
            let mut f = Fixture::new();
            let nested_body = f.child(&f.body.clone(), "nested_body");
            let nested = f.registry.register_loop(&f.body, key("nested")).unwrap();
            let size = CObjectType::scalar(CScalarType::Size);
            let counter = f
                .registry
                .register_local(&f.body, key("inner_counter"), size.clone())
                .unwrap();
            let bound = f
                .registry
                .register_local(
                    &f.body,
                    key("inner_bound"),
                    size.with_constness(CConstness::Const).unwrap(),
                )
                .unwrap();
            let values = f.expressions();
            let ast = f.statements();
            let condition = values
                .numeric_conversion(
                    CScalarType::Bool,
                    values
                        .binary(CBinaryOperator::Less, f.read(&counter), f.read(&bound))
                        .unwrap(),
                )
                .unwrap();
            let step = ast
                .assign(
                    values.local(counter.clone()).unwrap(),
                    values
                        .binary(CBinaryOperator::Add, f.read(&counter), f.size(1))
                        .unwrap(),
                )
                .unwrap();
            let mut inner = vec![allocation(&f), step];
            if continuing {
                inner.push(ast.continue_statement(nested.clone()).unwrap());
            }
            let iteration = ast
                .counted_loop(
                    nested,
                    counter.clone(),
                    bound.clone(),
                    condition,
                    ast.block(nested_body, inner).unwrap(),
                )
                .unwrap();
            let mut outer = vec![f.declare(&counter, f.size(0)), f.declare(&bound, f.size(2))];
            if outer_after {
                outer.push(f.step());
            }
            outer.push(iteration);
            if !outer_after {
                outer.push(f.step());
            }
            check(&f, outer, !outer_after);
        }
    }
}
