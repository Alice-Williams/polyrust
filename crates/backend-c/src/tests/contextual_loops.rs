//! Loop zero-iteration edges, redeclaration kills and cleanup joins.

use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

#[test]
fn a_loop_body_write_does_not_initialize_the_zero_iteration_exit() {
    for initially_initialized in [true, false] {
        let (mut registry, file, function, scope) = fixture();
        let body_scope = registry
            .register_scope(&function, Some(&scope), key("body"))
            .unwrap();
        let loop_id = registry.register_loop(&scope, key("iteration")).unwrap();
        let size = CObjectType::scalar(CScalarType::Size);
        let counter = registry
            .register_local(&scope, key("counter"), size.clone())
            .unwrap();
        let bound = registry
            .register_local(
                &scope,
                key("bound"),
                size.with_constness(CConstness::Const).unwrap(),
            )
            .unwrap();
        let output = registry
            .register_local(&scope, key("output"), CObjectType::scalar(CScalarType::I32))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let place = values.local(output.clone()).unwrap();
        let body = ast
            .block(
                body_scope,
                vec![
                    ast.assign(place.clone(), int(&values)).unwrap(),
                    ast.continue_statement(loop_id.clone()).unwrap(),
                ],
            )
            .unwrap();
        let iteration = ast
            .counted_loop(
                loop_id,
                counter.clone(),
                bound.clone(),
                values.literal(CLiteral::Bool(false)).unwrap(),
                body,
            )
            .unwrap();
        let zero = values
            .expression_initializer(
                values
                    .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                    .unwrap(),
            )
            .unwrap();
        let body = vec![
            ast.declare(
                output,
                initially_initialized.then(|| values.expression_initializer(int(&values)).unwrap()),
            )
            .unwrap(),
            ast.declare(counter, Some(zero.clone())).unwrap(),
            ast.declare(bound, Some(zero)).unwrap(),
            iteration,
            ast.discard(values.read(place).unwrap()).unwrap(),
        ];
        let result = registry.check_context(&[package(&registry, file, function, scope, body)]);
        if initially_initialized {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CContextError::UninitializedRead));
        }
    }
}

#[test]
fn a_cleanup_jump_does_not_inherit_initialization_from_the_path_it_skips() {
    for initialize_before_jump in [true, false] {
        let (mut registry, file, function, scope) = fixture();
        let local = registry
            .register_local(&scope, key("slot"), CObjectType::scalar(CScalarType::I32))
            .unwrap();
        let label = registry
            .register_cleanup_exit(&scope, key("cleanup"))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let place = values.local(local.clone()).unwrap();
        let mut body = vec![ast.declare(local, None).unwrap()];
        let assignment = ast.assign(place.clone(), int(&values)).unwrap();
        if initialize_before_jump {
            body.push(assignment.clone());
        }
        body.push(ast.cleanup_jump(label.clone()).unwrap());
        body.push(assignment);
        body.push(
            ast.label(label, ast.discard(values.read(place).unwrap()).unwrap())
                .unwrap(),
        );
        let result = registry.check_context(&[package(&registry, file, function, scope, body)]);
        if initialize_before_jump {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CContextError::UninitializedRead));
        }
    }
}
