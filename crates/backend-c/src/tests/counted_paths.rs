//! Finite update-count states distinguish each branch and exact exit edge.
use super::contextual_reconstruction::key;
use super::counted_fixture::Fixture;
use super::*;

#[test]
fn missing_and_double_steps_are_not_repaired_by_rendering() {
    let f = Fixture::new();
    assert_eq!(f.run(vec![]), Err(CSafetyError::MissingLoopStep));
    assert_eq!(
        f.run(vec![f.step(), f.step()]),
        Err(CSafetyError::RepeatedLoopStep)
    );
}

#[test]
fn continue_requires_a_preceding_step_and_cannot_count_unreachable_following_steps() {
    let f = Fixture::new();
    let next = f
        .statements()
        .continue_statement(f.identity.clone())
        .unwrap();
    f.run(vec![f.step(), next.clone()]).unwrap();
    assert_eq!(
        f.run(vec![next, f.step()]),
        Err(CSafetyError::MissingLoopStep)
    );
}

#[test]
fn unknown_branches_require_one_step_on_each_continuing_path() {
    for then_count in 0..=2 {
        for else_count in 0..=2 {
            let mut f = Fixture::new();
            let then_scope = f.child(&f.body.clone(), "then_branch");
            let else_scope = f.child(&f.body.clone(), "else_branch");
            let values = f.expressions();
            let ast = f.statements();
            // This is runtime-dependent to the structural path kernel; it
            // cannot select a branch from the fixture's bound initializer.
            let condition = values
                .numeric_conversion(CScalarType::Bool, f.read(&f.bound))
                .unwrap();
            let branch = ast
                .if_statement(
                    condition,
                    ast.block(then_scope, vec![f.step(); then_count]).unwrap(),
                    ast.block(else_scope, vec![f.step(); else_count]).unwrap(),
                )
                .unwrap();
            let result = f.run(vec![branch]);
            if then_count == 1 && else_count == 1 {
                result.unwrap();
            } else {
                assert!(matches!(
                    result,
                    Err(CSafetyError::MissingLoopStep | CSafetyError::RepeatedLoopStep)
                ));
            }
        }
    }
}

#[test]
fn exact_constant_predicates_can_exclude_a_missing_step_path() {
    for selected in [false, true] {
        let mut f = Fixture::new();
        let then_scope = f.child(&f.body.clone(), "then_branch");
        let else_scope = f.child(&f.body.clone(), "else_branch");
        let ast = f.statements();
        let branch = ast
            .if_statement(
                f.expressions().literal(CLiteral::Bool(selected)).unwrap(),
                ast.block(then_scope, if selected { vec![f.step()] } else { vec![] })
                    .unwrap(),
                ast.block(else_scope, if selected { vec![] } else { vec![f.step()] })
                    .unwrap(),
            )
            .unwrap();
        f.run(vec![branch]).unwrap();
    }
}

#[test]
fn early_break_return_and_cleanup_need_no_update_but_do_not_allow_two() {
    for kind in 0..3 {
        let mut f = Fixture::new();
        let cleanup = (kind == 2).then(|| {
            f.registry
                .register_cleanup_exit(&f.scope, key("cleanup"))
                .unwrap()
        });
        let ast = f.statements();
        let exit = match kind {
            0 => ast
                .break_statement(CBreakTarget::Loop(f.identity.clone()))
                .unwrap(),
            1 => ast.return_statement(None).unwrap(),
            _ => ast.cleanup_jump(cleanup.clone().unwrap()).unwrap(),
        };
        for count in 0..=2 {
            let mut body = vec![f.step(); count];
            body.push(exit.clone());
            let mut root = vec![
                f.declare(&f.counter, f.size(0)),
                f.declare(&f.bound, f.size(3)),
                f.iteration(body),
            ];
            if let Some(cleanup) = &cleanup {
                root.push(ast.label(cleanup.clone(), ast.empty()).unwrap());
            }
            let result = f.check(root);
            if count < 2 {
                result.unwrap();
            } else {
                assert_eq!(result, Err(CSafetyError::RepeatedLoopStep));
            }
        }
    }
}

#[test]
fn cleanup_within_the_loop_cannot_skip_the_only_update() {
    let mut f = Fixture::new();
    let label = f
        .registry
        .register_cleanup_exit(&f.body, key("after_step"))
        .unwrap();
    let ast = f.statements();
    assert_eq!(
        f.run(vec![
            ast.cleanup_jump(label.clone()).unwrap(),
            f.step(),
            ast.label(label, ast.empty()).unwrap()
        ]),
        Err(CSafetyError::MissingLoopStep)
    );
}
