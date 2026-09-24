//! Public certification, not just syntax, checks every annotated child.
use super::*;

fn admitted(index: usize, value: JavaExpr) -> bool {
    fixture::admitted(f::package(820, vec![fixture::function(index, value)]))
}

#[test]
fn wrapping_subtraction_requires_exact_types_and_precedence() {
    for (index, width) in fixture::WIDTHS.into_iter().enumerate() {
        let base = subtract(
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
        let bad_types = [
            JavaPrimitive::Byte,
            JavaPrimitive::Char,
            JavaPrimitive::Int,
            JavaPrimitive::Long,
            JavaPrimitive::Double,
            JavaPrimitive::Boolean,
            JavaPrimitive::Void,
        ]
        .into_iter()
        .filter(|other| *other != width)
        .map(JavaType::primitive)
        .chain([
            JavaType::Boxed(JavaPrimitive::Int),
            JavaType::Boxed(JavaPrimitive::Long),
            JavaType::known(JavaKnownType::String),
        ]);
        for other in bad_types {
            for side in 0..3 {
                let mut value = base.clone();
                let JavaExprKind::Binary { left, right, .. } = &mut value.kind else {
                    unreachable!()
                };
                match side {
                    0 => left.ty = other.clone(),
                    1 => right.ty = other.clone(),
                    _ => value.ty = other.clone(),
                }
                assert!(!admitted(index, value), "{width:?} {other:?} side {side}");
            }
        }
    }
}

#[test]
fn wrapping_subtraction_recursively_rejects_unadmitted_shapes_and_depth() {
    for (index, width) in fixture::WIDTHS.into_iter().enumerate() {
        let one = || fixture::literal(width, 1);
        for operator in [
            JavaBinaryOperator::Divide,
            JavaBinaryOperator::Remainder,
            JavaBinaryOperator::ShiftLeft,
            JavaBinaryOperator::ShiftRight,
        ] {
            let mut bad = fixture::binary(operator, width, one(), one());
            bad.precedence = match operator {
                JavaBinaryOperator::ShiftLeft | JavaBinaryOperator::ShiftRight => {
                    JavaPrecedence::Shift
                }
                _ => JavaPrecedence::Multiplicative,
            };
            assert!(!admitted(index, bad.clone()));
            for (left, right) in [(bad.clone(), one()), (one(), bad)] {
                assert!(!admitted(index, subtract(width, left, right)));
            }
        }
        let cast = JavaExpr {
            ty: JavaType::primitive(width),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Cast {
                target: JavaType::primitive(width),
                value: Box::new(one()),
            },
        };
        for (left, right) in [(cast.clone(), one()), (one(), cast)] {
            assert!(!admitted(index, subtract(width, left, right)));
        }
        for side in [false, true] {
            let mut value = one();
            for _ in 0..16 {
                value = if side {
                    subtract(width, one(), value)
                } else {
                    subtract(width, value, one())
                };
            }
            assert!(admitted(index, value.clone()));
            for _ in 0..128 {
                value = if side {
                    subtract(width, one(), value)
                } else {
                    subtract(width, value, one())
                };
            }
            assert!(!admitted(index, value));
        }
    }
}

#[test]
fn wrapping_subtraction_recursively_checks_original_import_authority_and_arity() {
    let owners = chain();
    for (index, target) in owners[0].functions().cloned().enumerate() {
        let width = fixture::WIDTHS[index];
        for side in [false, true] {
            for register in [false, true] {
                for arity in [1, 2, 3] {
                    let (scope, callable) =
                        JavaDependencyScope::new().import(target.clone()).unwrap();
                    let call = JavaExpr {
                        ty: JavaType::primitive(width),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Call {
                            callable: JavaCallableRef::Dependency(callable),
                            receiver: None,
                            arguments: vec![fixture::literal(width, 1); arity],
                        },
                    };
                    let zero = fixture::literal(width, 0);
                    let (left, right) = if side { (call, zero) } else { (zero, call) };
                    let package = f::package_with_dependencies(
                        821,
                        vec![fixture::function(index, subtract(width, left, right))],
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
