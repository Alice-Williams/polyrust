use super::*;

fn admitted(index: usize, value: JavaExpr) -> bool {
    fixture::admitted(f::package(810, vec![fixture::function(index, value)]))
}

#[test]
fn wrapping_addition_requires_exact_types_and_precedence() {
    for (index, width) in fixture::WIDTHS.into_iter().enumerate() {
        let base = fixture::add(
            width,
            fixture::literal(width, 1),
            fixture::literal(width, 2),
        );
        assert!(admitted(index, base.clone()));
        for precedence in [
            JavaPrecedence::Assignment,
            JavaPrecedence::Conditional,
            JavaPrecedence::LogicalOr,
            JavaPrecedence::LogicalAnd,
            JavaPrecedence::BitOr,
            JavaPrecedence::BitXor,
            JavaPrecedence::BitAnd,
            JavaPrecedence::Equality,
            JavaPrecedence::Relational,
            JavaPrecedence::Shift,
            JavaPrecedence::Additive,
            JavaPrecedence::Multiplicative,
            JavaPrecedence::Unary,
            JavaPrecedence::Primary,
        ] {
            let mut value = base.clone();
            value.precedence = precedence;
            assert_eq!(
                admitted(index, value),
                precedence == JavaPrecedence::Additive
            );
        }
        for other in [
            JavaPrimitive::Byte,
            JavaPrimitive::Char,
            JavaPrimitive::Int,
            JavaPrimitive::Long,
            JavaPrimitive::Double,
            JavaPrimitive::Boolean,
            JavaPrimitive::Void,
        ] {
            if other == width {
                continue;
            }
            for side in 0..3 {
                let mut value = base.clone();
                let JavaExprKind::Binary { left, right, .. } = &mut value.kind else {
                    unreachable!()
                };
                match side {
                    0 => left.ty = JavaType::primitive(other),
                    1 => right.ty = JavaType::primitive(other),
                    _ => value.ty = JavaType::primitive(other),
                }
                assert!(!admitted(index, value), "{width:?} {other:?} side {side}");
            }
        }
        for side in 0..3 {
            let mut value = base.clone();
            let JavaExprKind::Binary { left, right, .. } = &mut value.kind else {
                unreachable!()
            };
            match side {
                0 => left.ty = JavaType::Boxed(width),
                1 => right.ty = JavaType::Boxed(width),
                _ => value.ty = JavaType::Boxed(width),
            }
            assert!(!admitted(index, value));
        }
    }
}

#[test]
fn wrapping_addition_does_not_admit_other_integer_arithmetic_or_hidden_casts() {
    for (index, width) in fixture::WIDTHS.into_iter().enumerate() {
        for operator in [
            JavaBinaryOperator::Divide,
            JavaBinaryOperator::Remainder,
            JavaBinaryOperator::ShiftLeft,
            JavaBinaryOperator::ShiftRight,
        ] {
            let mut value = fixture::add(
                width,
                fixture::literal(width, 1),
                fixture::literal(width, 2),
            );
            value.precedence = match operator {
                JavaBinaryOperator::ShiftLeft | JavaBinaryOperator::ShiftRight => {
                    JavaPrecedence::Shift
                }
                _ => JavaPrecedence::Multiplicative,
            };
            let JavaExprKind::Binary {
                operator: actual, ..
            } = &mut value.kind
            else {
                unreachable!()
            };
            *actual = operator;
            assert!(!admitted(index, value.clone()));
            for side in [false, true] {
                let (left, right) = if side {
                    (value.clone(), fixture::literal(width, 3))
                } else {
                    (fixture::literal(width, 3), value.clone())
                };
                assert!(!admitted(index, fixture::add(width, left, right)));
            }
        }
        let cast = JavaExpr {
            ty: JavaType::primitive(width),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Cast {
                target: JavaType::primitive(width),
                value: Box::new(fixture::literal(width, 1)),
            },
        };
        assert!(!admitted(
            index,
            fixture::add(width, cast, fixture::literal(width, 1))
        ));
        let mut value = fixture::literal(width, 0);
        for _ in 0..16 {
            value = fixture::add(width, value, fixture::literal(width, 1));
        }
        assert!(admitted(index, value.clone()));
        for _ in 0..128 {
            value = fixture::add(width, value, fixture::literal(width, 1));
        }
        assert!(!admitted(index, value));
    }
}

#[test]
fn wrapping_addition_recursively_checks_original_import_authority_and_arity() {
    let owners = fixture::chain();
    for (index, target) in owners[0].functions().cloned().enumerate() {
        let width = fixture::WIDTHS[index];
        for side in [false, true] {
            for register in [false, true] {
                for arity in [1, 2, 3] {
                    let (scope, callable) = JavaDependencyScope::new().import(target.clone());
                    let call = JavaExpr {
                        ty: JavaType::primitive(width),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Call {
                            callable: JavaCallableRef::Dependency(callable),
                            receiver: None,
                            arguments: vec![fixture::literal(width, 1); arity],
                        },
                    };
                    let (left, right) = if side {
                        (call, fixture::literal(width, 0))
                    } else {
                        (fixture::literal(width, 0), call)
                    };
                    let package = f::package_with_dependencies(
                        811,
                        vec![fixture::function(index, fixture::add(width, left, right))],
                        if register {
                            scope.finish()
                        } else {
                            JavaDependencyScope::new().finish()
                        },
                    );
                    assert_eq!(fixture::admitted(package), register && arity == 2);
                }
            }
        }
    }
}
