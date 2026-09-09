//! Exact syntax, declaration identities, full write inventory and aliases.
use super::contextual_reconstruction::key;
use super::counted_fixture::Fixture;
use super::*;

#[test]
fn zero_one_many_and_size_max_bounds_have_the_same_structural_proof() {
    for bound in [0, 1, 7, u64::MAX - 1, u64::MAX] {
        let f = Fixture::new();
        f.check(vec![
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(bound)),
            f.iteration(vec![f.step()]),
        ])
        .unwrap();
    }
}

#[test]
fn bound_snapshot_may_be_a_runtime_read_not_a_literal() {
    let mut f = Fixture::new();
    let source = f
        .registry
        .register_local(
            &f.scope,
            key("runtime_bound"),
            CObjectType::scalar(CScalarType::Size),
        )
        .unwrap();
    f.check(vec![
        f.declare(&source, f.size(8)),
        f.declare(&f.counter, f.size(0)),
        f.declare(&f.bound, f.read(&source)),
        f.iteration(vec![f.step()]),
    ])
    .unwrap();
}

#[test]
fn changed_initialization_and_condition_trees_are_rejected() {
    let f = Fixture::new();
    for init in [
        f.size(1),
        f.expressions()
            .literal(CLiteral::Unsigned(CUnsignedLiteral::U64(0)))
            .unwrap(),
    ] {
        assert_eq!(
            f.check(vec![
                f.declare(&f.counter, init),
                f.declare(&f.bound, f.size(3)),
                f.iteration(vec![f.step()])
            ]),
            Err(CSafetyError::InvalidCountedLoop)
        );
    }
    let values = f.expressions();
    let mut conditions = vec![values.literal(CLiteral::Bool(true)).unwrap()];
    for (operator, left, right) in [
        (
            CBinaryOperator::LessEqual,
            f.read(&f.counter),
            f.read(&f.bound),
        ),
        (
            CBinaryOperator::Greater,
            f.read(&f.counter),
            f.read(&f.bound),
        ),
        (CBinaryOperator::Less, f.read(&f.bound), f.read(&f.counter)),
        (
            CBinaryOperator::Less,
            f.read(&f.counter),
            f.read(&f.counter),
        ),
        (CBinaryOperator::Less, f.read(&f.counter), f.size(3)),
    ] {
        conditions.push(
            values
                .numeric_conversion(
                    CScalarType::Bool,
                    values.binary(operator, left, right).unwrap(),
                )
                .unwrap(),
        );
    }
    for condition in conditions {
        let mut iteration = f.iteration(vec![f.step()]);
        let CStatementKind::BoundedLoop {
            condition: actual, ..
        } = &mut iteration.kind
        else {
            unreachable!()
        };
        *actual = condition;
        let source = f.source(vec![
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(3)),
            iteration,
        ]);
        f.registry
            .check_sequencing_and_control(std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(
            f.registry.check_counted_loops(&[source]),
            Err(CSafetyError::InvalidCountedLoop)
        );
    }
}

#[test]
fn all_counter_writes_must_be_actual_canonical_owned_steps() {
    let f = Fixture::new();
    let values = f.expressions();
    for (operator, left, right) in [
        (CBinaryOperator::Add, f.read(&f.counter), f.size(0)),
        (CBinaryOperator::Add, f.read(&f.counter), f.size(2)),
        (CBinaryOperator::Subtract, f.read(&f.counter), f.size(1)),
        (CBinaryOperator::Add, f.read(&f.bound), f.size(1)),
        (CBinaryOperator::Add, f.size(1), f.read(&f.counter)),
    ] {
        let wrong = f
            .statements()
            .assign(
                values.local(f.counter.clone()).unwrap(),
                values.binary(operator, left, right).unwrap(),
            )
            .unwrap();
        assert_eq!(f.run(vec![wrong]), Err(CSafetyError::InvalidLoopMutation));
    }
    for before in [true, false] {
        let mut body = vec![
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(3)),
        ];
        if before {
            body.push(f.step());
        }
        body.push(f.iteration(vec![f.step()]));
        if !before {
            body.push(f.step());
        }
        assert_eq!(f.check(body), Err(CSafetyError::InvalidLoopMutation));
    }
}

#[test]
fn direct_wrapped_and_unreachable_addresses_cannot_hide_counter_or_bound_escape() {
    for bound in [false, true] {
        for unreachable in [false, true] {
            let mut f = Fixture::new();
            let dead = f.child(&f.body.clone(), "dead_branch");
            let live = f.child(&f.body.clone(), "live_branch");
            let values = f.expressions();
            let address = values
                .address_of(
                    values
                        .local(if bound {
                            f.bound.clone()
                        } else {
                            f.counter.clone()
                        })
                        .unwrap(),
                )
                .unwrap();
            let pointer = values
                .object_to_void(
                    CObjectType::pointer(CPointerTarget::Void(if bound {
                        CConstness::Const
                    } else {
                        CConstness::Unqualified
                    })),
                    address,
                )
                .unwrap();
            let ast = f.statements();
            let branch = ast
                .if_statement(
                    values.literal(CLiteral::Bool(!unreachable)).unwrap(),
                    ast.block(dead, vec![ast.discard(pointer).unwrap()])
                        .unwrap(),
                    ast.block(live, vec![]).unwrap(),
                )
                .unwrap();
            assert_eq!(
                f.run(vec![branch, f.step()]),
                Err(CSafetyError::LoopAddressEscape)
            );
        }
    }
}

#[test]
fn a_counter_cannot_be_reused_by_sequential_loops() {
    let mut f = Fixture::new();
    let other = f.registry.register_loop(&f.scope, key("another")).unwrap();
    let body = f.child(&f.scope.clone(), "second_body");
    let ast = f.statements();
    let second = ast
        .counted_loop(
            other,
            f.counter.clone(),
            f.bound.clone(),
            f.condition(),
            ast.block(body, vec![f.step()]).unwrap(),
        )
        .unwrap();
    assert_eq!(
        f.check(vec![
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(3)),
            f.iteration(vec![f.step()]),
            second
        ]),
        Err(CSafetyError::SharedLoopCounter)
    );
}

#[test]
fn declaration_reordering_and_owner_substitution_still_fail_context() {
    let f = Fixture::new();
    assert!(matches!(
        f.check(vec![
            f.declare(&f.bound, f.size(3)),
            f.iteration(vec![f.step()]),
            f.declare(&f.counter, f.size(0))
        ]),
        Err(CSafetyError::Context(_))
    ));
    let mut iteration = f.iteration(vec![f.step()]);
    let CStatementKind::BoundedLoop { progress, .. } = &mut iteration.kind else {
        unreachable!()
    };
    progress.counter = f.bound.clone();
    assert!(matches!(
        f.check(vec![
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(3)),
            iteration
        ]),
        Err(CSafetyError::Context(_))
    ));
}
