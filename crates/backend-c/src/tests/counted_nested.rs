//! Nested scopes retain exact loop ownership and switch/Continue destinations.
use super::contextual_reconstruction::key;
use super::counted_fixture::Fixture;
use super::*;

#[test]
fn nested_counted_cycles_preserve_outer_step_count_and_reinitialize_inner_locals() {
    for outer_first in [false, true] {
        for inner_continue in [false, true] {
            for mutate_outer in [false, true] {
                let mut f = Fixture::new();
                let nested_body = f.child(&f.body.clone(), "nested_body");
                let nested = f
                    .registry
                    .register_loop(&f.body, key("nested_loop"))
                    .unwrap();
                let size = CObjectType::scalar(CScalarType::Size);
                let counter = f
                    .registry
                    .register_local(&f.body, key("nested_counter"), size.clone())
                    .unwrap();
                let bound = f
                    .registry
                    .register_local(
                        &f.body,
                        key("nested_bound"),
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
                let mut inner = vec![step];
                if mutate_outer {
                    inner.push(f.step());
                }
                if inner_continue {
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
                let mut outer = vec![
                    f.declare(&counter, f.size(0)),
                    f.declare(&bound, f.read(&f.bound)),
                ];
                if outer_first {
                    outer.push(f.step());
                }
                outer.push(iteration);
                if !outer_first {
                    outer.push(f.step());
                }
                let result = f.run(outer);
                if mutate_outer {
                    assert_eq!(result, Err(CSafetyError::InvalidLoopMutation));
                } else {
                    result.unwrap();
                }
            }
        }
    }
}

#[test]
fn switch_continue_counts_its_branch_step_but_switch_break_returns_to_outer_step() {
    for missing in [false, true] {
        let mut f = Fixture::new();
        let case = f.child(&f.body.clone(), "case_scope");
        let fallback = f.child(&f.body.clone(), "fallback_scope");
        let switch = f
            .registry
            .register_switch(&f.body, key("selection"))
            .unwrap();
        let ast = f.statements();
        let mut branch = vec![];
        if !missing {
            branch.push(f.step());
        }
        branch.push(ast.continue_statement(f.identity.clone()).unwrap());
        let selection = ast
            .switch_statement(
                switch.clone(),
                f.read(&f.bound),
                vec![
                    ast.switch_arm(
                        vec![CCaseConstant::Unsigned(CUnsignedLiteral::Size(1))],
                        ast.block(case, branch).unwrap(),
                    )
                    .unwrap(),
                ],
                ast.block(
                    fallback,
                    vec![ast.break_statement(CBreakTarget::Switch(switch)).unwrap()],
                )
                .unwrap(),
            )
            .unwrap();
        let result = f.run(vec![selection, f.step()]);
        if missing {
            assert_eq!(result, Err(CSafetyError::MissingLoopStep));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn branch_local_exit_does_not_require_a_step_on_the_exiting_branch() {
    let mut f = Fixture::new();
    let finish = f.child(&f.body.clone(), "finish_scope");
    let next = f.child(&f.body.clone(), "next_scope");
    let values = f.expressions();
    let ast = f.statements();
    let branch = ast
        .if_statement(
            values
                .numeric_conversion(CScalarType::Bool, f.read(&f.bound))
                .unwrap(),
            ast.block(
                finish,
                vec![
                    ast.break_statement(CBreakTarget::Loop(f.identity.clone()))
                        .unwrap(),
                ],
            )
            .unwrap(),
            ast.block(
                next,
                vec![
                    f.step(),
                    ast.continue_statement(f.identity.clone()).unwrap(),
                ],
            )
            .unwrap(),
        )
        .unwrap();
    f.run(vec![branch]).unwrap();
}

#[test]
fn ordinary_nested_blocks_and_labels_retain_actual_step_occurrences() {
    let mut f = Fixture::new();
    let scope = f.child(&f.body.clone(), "ordinary");
    let label = f
        .registry
        .register_cleanup_exit(&scope, key("update_label"))
        .unwrap();
    let ast = f.statements();
    let block = ast
        .nested_block(
            ast.block(scope, vec![ast.label(label, f.step()).unwrap()])
                .unwrap(),
        )
        .unwrap();
    f.run(vec![block]).unwrap();
}

#[test]
fn separate_counters_may_share_one_immutable_bound() {
    let mut f = Fixture::new();
    let identity = f
        .registry
        .register_loop(&f.scope, key("other_loop"))
        .unwrap();
    let body = f.child(&f.scope.clone(), "other_body");
    let counter = f
        .registry
        .register_local(
            &f.scope,
            key("other_counter"),
            CObjectType::scalar(CScalarType::Size),
        )
        .unwrap();
    let values = f.expressions();
    let ast = f.statements();
    let step = ast
        .assign(
            values.local(counter.clone()).unwrap(),
            values
                .binary(CBinaryOperator::Add, f.read(&counter), f.size(1))
                .unwrap(),
        )
        .unwrap();
    let condition = values
        .numeric_conversion(
            CScalarType::Bool,
            values
                .binary(CBinaryOperator::Less, f.read(&counter), f.read(&f.bound))
                .unwrap(),
        )
        .unwrap();
    let second = ast
        .counted_loop(
            identity,
            counter.clone(),
            f.bound.clone(),
            condition,
            ast.block(body, vec![step]).unwrap(),
        )
        .unwrap();
    f.check(vec![
        f.declare(&f.counter, f.size(0)),
        f.declare(&counter, f.size(0)),
        f.declare(&f.bound, f.size(3)),
        f.iteration(vec![f.step()]),
        second,
    ])
    .unwrap();
}
