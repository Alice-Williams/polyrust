//! The dependency reader itself must reject malformed arithmetic annotations.
use super::*;
fn literal(ty: JavaPrimitive) -> JavaExpr {
    let payload = match ty {
        JavaPrimitive::Double => {
            JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(1).unwrap())
        }
        JavaPrimitive::Int => JavaLiteral::I32(1),
        JavaPrimitive::Long => JavaLiteral::I64(1),
        JavaPrimitive::Boolean => JavaLiteral::Boolean(true),
        _ => unreachable!(),
    };
    JavaExpr::literal(JavaType::primitive(ty), payload)
}
fn admitted(value: &JavaExpr) -> bool {
    let mut budget = Budget::new();
    Reader {
        methods: &BTreeMap::new(),
        records: &BTreeMap::new(),
        constants: &BTreeMap::new(),
        budget: &mut budget,
        calls: BTreeSet::new(),
        imported_height: 0,
        mutable_bools: BTreeSet::new(),
    }
    .expression(value, 0)
    .is_ok()
}
#[test]
fn arithmetic_reader_requires_exact_operands_results_and_precedence() {
    let types = [
        JavaPrimitive::Double,
        JavaPrimitive::Int,
        JavaPrimitive::Long,
        JavaPrimitive::Boolean,
    ];
    for left in types {
        for right in types {
            for result in types {
                for operator in [
                    JavaBinaryOperator::Add,
                    JavaBinaryOperator::Subtract,
                    JavaBinaryOperator::Multiply,
                    JavaBinaryOperator::Divide,
                    JavaBinaryOperator::Remainder,
                ] {
                    for precedence in [
                        JavaPrecedence::Primary,
                        JavaPrecedence::Additive,
                        JavaPrecedence::Multiplicative,
                    ] {
                        let value = JavaExpr {
                            ty: JavaType::primitive(result),
                            precedence,
                            kind: JavaExprKind::Binary {
                                operator,
                                left: Box::new(literal(left)),
                                right: Box::new(literal(right)),
                            },
                        };
                        let expected_precedence = match operator {
                            JavaBinaryOperator::Add | JavaBinaryOperator::Subtract => {
                                JavaPrecedence::Additive
                            }
                            _ => JavaPrecedence::Multiplicative,
                        };
                        assert_eq!(
                            admitted(&value),
                            (left == JavaPrimitive::Double
                                || (matches!(left, JavaPrimitive::Int | JavaPrimitive::Long)
                                    && matches!(
                                        operator,
                                        JavaBinaryOperator::Add | JavaBinaryOperator::Subtract
                                    )))
                                && right == left
                                && result == left
                                && precedence == expected_precedence,
                            "{left:?} {operator:?} {right:?} -> {result:?}, {precedence:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn integer_additive_operations_do_not_admit_boxing_or_strings() {
    for operator in [JavaBinaryOperator::Add, JavaBinaryOperator::Subtract] {
        for ty in [
            JavaType::Boxed(JavaPrimitive::Int),
            JavaType::Boxed(JavaPrimitive::Long),
            JavaType::known(crate::ast::JavaKnownType::String),
        ] {
            let value = JavaExpr {
                ty: ty.clone(),
                precedence: JavaPrecedence::Additive,
                kind: JavaExprKind::Binary {
                    operator,
                    left: Box::new(JavaExpr::local(
                        ty.clone(),
                        JavaIdentifier::new("left").unwrap(),
                    )),
                    right: Box::new(JavaExpr::local(ty, JavaIdentifier::new("right").unwrap())),
                },
            };
            assert!(!admitted(&value));
        }
    }
}
