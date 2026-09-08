//! Definite initialization respects conditional evaluation, not just syntax.

use super::contextual_reconstruction::{fixture, key, package};
use super::*;

#[test]
fn short_circuit_and_conditional_arms_only_read_when_selected() {
    for selected in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let local = registry
            .register_local(&scope, key("value"), CObjectType::scalar(CScalarType::Bool))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let read = values.read(values.local(local.clone()).unwrap()).unwrap();
        let literal = |value| values.literal(CLiteral::Bool(value)).unwrap();
        let expressions = [
            values
                .binary(CBinaryOperator::LogicalAnd, literal(selected), read.clone())
                .unwrap(),
            values
                .binary(CBinaryOperator::LogicalOr, literal(!selected), read.clone())
                .unwrap(),
            values
                .conditional(literal(selected), read.clone(), literal(false))
                .unwrap(),
            values
                .conditional(literal(!selected), literal(false), read)
                .unwrap(),
        ];
        for value in expressions {
            let source = package(
                &registry,
                file.clone(),
                function.clone(),
                scope.clone(),
                vec![
                    ast.declare(local.clone(), None).unwrap(),
                    ast.discard(value).unwrap(),
                ],
            );
            let result = registry.check_context(&[source]);
            if selected {
                assert_eq!(result, Err(CContextError::UninitializedRead));
            } else {
                result.unwrap();
            }
        }
    }
}

#[test]
fn skipped_evaluation_never_skips_lexical_or_child_type_checking() {
    let (mut registry, file, function, scope) = fixture();
    let local = registry
        .register_local(&scope, key("late"), CObjectType::scalar(CScalarType::Bool))
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let skipped = values
        .binary(
            CBinaryOperator::LogicalAnd,
            values.literal(CLiteral::Bool(false)).unwrap(),
            values.read(values.local(local.clone()).unwrap()).unwrap(),
        )
        .unwrap();
    let source = package(
        &registry,
        file.clone(),
        function.clone(),
        scope.clone(),
        vec![
            ast.discard(skipped.clone()).unwrap(),
            ast.declare(local.clone(), None).unwrap(),
        ],
    );
    assert_eq!(
        registry.check_context(&[source]),
        Err(CContextError::InvisibleBinding)
    );
    let mut bad = skipped;
    let CValueKind::Binary { right, .. } = &mut bad.kind else {
        unreachable!()
    };
    right.ty = CObjectType::scalar(CScalarType::I32);
    let source = package(
        &registry,
        file,
        function,
        scope,
        vec![ast.declare(local, None).unwrap(), ast.discard(bad).unwrap()],
    );
    assert!(registry.check_context(&[source]).is_err());
}

#[test]
fn an_unknown_condition_keeps_both_possible_read_paths() {
    let (mut registry, file, function, scope) = fixture();
    let ty = CObjectType::scalar(CScalarType::Bool);
    let condition = registry
        .register_local(&scope, key("condition"), ty.clone())
        .unwrap();
    let local = registry
        .register_local(&scope, key("uninitialized"), ty)
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let condition_read = values
        .read(values.local(condition.clone()).unwrap())
        .unwrap();
    let bad_read = values.read(values.local(local.clone()).unwrap()).unwrap();
    let expressions = [
        values
            .binary(
                CBinaryOperator::LogicalAnd,
                condition_read.clone(),
                bad_read.clone(),
            )
            .unwrap(),
        values
            .binary(
                CBinaryOperator::LogicalOr,
                condition_read.clone(),
                bad_read.clone(),
            )
            .unwrap(),
        values
            .conditional(
                condition_read,
                bad_read,
                values.literal(CLiteral::Bool(false)).unwrap(),
            )
            .unwrap(),
    ];
    for value in expressions {
        let body = vec![
            ast.declare(
                condition.clone(),
                Some(
                    values
                        .expression_initializer(values.literal(CLiteral::Bool(false)).unwrap())
                        .unwrap(),
                ),
            )
            .unwrap(),
            ast.declare(local.clone(), None).unwrap(),
            ast.discard(value).unwrap(),
        ];
        // 02C does not constant-propagate mutable local values; either outcome
        // remains possible until a separate range/value proof establishes it.
        assert_eq!(
            registry.check_context(&[package(
                &registry,
                file.clone(),
                function.clone(),
                scope.clone(),
                body
            )]),
            Err(CContextError::UninitializedRead)
        );
    }
}
