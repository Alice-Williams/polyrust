//! Public certification keeps exact remainder shape and original call authority.
use super::*;
fn admitted(value: JavaExpr) -> bool {
    let function = fixture::function(0, JavaBlock::new(vec![JavaStmt::Return(Some(value))]));
    let package = f::package(610, vec![function]);
    let Ok(verified) = verify_unresolved_package(&JavaDialect, package) else {
        return false;
    };
    let Ok(linked) = TargetLinker::new(JavaDialect).link_ast(&verified) else {
        return false;
    };
    let Ok(ready) = certify_resolved_package(&JavaDialect, linked) else {
        return false;
    };
    JavaDependencyApi::from_certificate(ready).is_ok()
}
#[test]
fn remainder_requires_exact_type_operator_and_precedence() {
    let operator = JavaBinaryOperator::Remainder;
    let value = fixture::binary(operator, fixture::literal(0), fixture::literal(1));
    assert!(admitted(value.clone()));
    for precedence in [
        JavaPrecedence::Primary,
        JavaPrecedence::Unary,
        JavaPrecedence::Additive,
        JavaPrecedence::Multiplicative,
    ] {
        let mut changed = value.clone();
        changed.precedence = precedence;
        assert_eq!(admitted(changed), precedence == value.precedence);
    }
    for side in 0..3 {
        let mut changed = value.clone();
        let JavaExprKind::Binary { left, right, .. } = &mut changed.kind else {
            panic!("binary")
        };
        match side {
            0 => **left = JavaExpr::literal(f::int(), JavaLiteral::I32(0)),
            1 => **right = JavaExpr::literal(f::int(), JavaLiteral::I32(0)),
            _ => changed.ty = f::int(),
        }
        assert!(!admitted(changed));
    }
}
#[test]
fn nested_remainder_cannot_hide_unregistered_or_wrong_arity_imports() {
    let owners = fixture::chain();
    let owner = owners[1].functions().next().unwrap().clone();
    for register in [false, true] {
        for arity in [1, 2, 3] {
            let (scope, callable) = JavaDependencyScope::new().import(owner.clone());
            let expression = fixture::binary(
                JavaBinaryOperator::Remainder,
                fixture::literal(0),
                JavaExpr {
                    ty: double(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Call {
                        callable: JavaCallableRef::Dependency(callable),
                        receiver: None,
                        arguments: vec![fixture::literal(0); arity],
                    },
                },
            );
            let function =
                fixture::function(0, JavaBlock::new(vec![JavaStmt::Return(Some(expression))]));
            let package = f::package_with_dependencies(
                611,
                vec![function],
                if register {
                    scope.finish()
                } else {
                    JavaDependencyScope::new().finish()
                },
            );
            let result = verify_unresolved_package(&JavaDialect, package);
            assert_eq!(result.is_ok(), register && arity == 2);
        }
    }
}

#[test]
fn remainder_recursively_rejects_unadmitted_operand_shapes() {
    let invalid_integer_operand = fixture::binary(
        JavaBinaryOperator::Remainder,
        JavaExpr::literal(f::int(), JavaLiteral::I32(0)),
        fixture::literal(1),
    );
    let cast = JavaExpr {
        ty: double(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: double(),
            value: Box::new(JavaExpr::literal(f::int(), JavaLiteral::I32(1))),
        },
    };
    for operand in [invalid_integer_operand, cast] {
        for left in [false, true] {
            let (a, b) = if left {
                (operand.clone(), fixture::literal(1))
            } else {
                (fixture::literal(1), operand.clone())
            };
            assert!(!admitted(fixture::binary(
                JavaBinaryOperator::Remainder,
                a,
                b
            )));
        }
    }
}

#[test]
fn floating_remainder_does_not_admit_integer_remainder() {
    for primitive in [JavaPrimitive::Int, JavaPrimitive::Long] {
        let ty = JavaType::primitive(primitive);
        let literal = match primitive {
            JavaPrimitive::Int => JavaLiteral::I32(1),
            JavaPrimitive::Long => JavaLiteral::I64(1),
            _ => unreachable!(),
        };
        let value = JavaExpr {
            ty: ty.clone(),
            precedence: JavaPrecedence::Multiplicative,
            kind: JavaExprKind::Binary {
                operator: JavaBinaryOperator::Remainder,
                left: Box::new(JavaExpr::literal(ty.clone(), literal.clone())),
                right: Box::new(JavaExpr::literal(ty.clone(), literal)),
            },
        };
        let mut function =
            fixture::function(0, JavaBlock::new(vec![JavaStmt::Return(Some(value))]));
        function.result = ty;
        let ready = f::certify(f::package(612, vec![function]));
        assert!(JavaDependencyApi::from_certificate(ready).is_err());
    }
}

#[test]
fn remainder_composes_in_both_arithmetic_operand_positions() {
    for operator in [
        JavaBinaryOperator::Add,
        JavaBinaryOperator::Subtract,
        JavaBinaryOperator::Multiply,
        JavaBinaryOperator::Divide,
        JavaBinaryOperator::Remainder,
    ] {
        let remainder = || {
            fixture::binary(
                JavaBinaryOperator::Remainder,
                fixture::literal(1),
                fixture::literal(2),
            )
        };
        assert!(admitted(fixture::binary(
            operator,
            remainder(),
            fixture::literal(1)
        )));
        assert!(admitted(fixture::binary(
            operator,
            fixture::literal(1),
            remainder()
        )));
    }
}
