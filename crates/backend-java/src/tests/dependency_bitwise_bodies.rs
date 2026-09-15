//! Private dependency-body checks independently enforce exact bitwise types.
use super::*;
use crate::ast::JavaPrecedence;

fn literal(ty: JavaPrimitive) -> JavaExpr {
    let literal = match ty {
        JavaPrimitive::Int => JavaLiteral::I32(1),
        JavaPrimitive::Long => JavaLiteral::I64(1),
        JavaPrimitive::Boolean => JavaLiteral::Boolean(true),
        _ => unreachable!(),
    };
    JavaExpr::literal(JavaType::primitive(ty), literal)
}
fn admitted(value: JavaExpr) -> bool {
    let mut budget = Budget::new();
    Reader {
        methods: &BTreeMap::new(),
        records: &BTreeMap::new(),
        budget: &mut budget,
        calls: BTreeSet::new(),
        imported_height: 0,
        mutable_bools: BTreeSet::new(),
    }
    .expression(&value, 0)
    .is_ok()
}

#[test]
fn exact_bitwise_operands_and_results_are_required_by_dependency_bodies() {
    let types = [
        JavaPrimitive::Int,
        JavaPrimitive::Long,
        JavaPrimitive::Boolean,
    ];
    for left in types {
        for result in types {
            for operator in [JavaUnaryOperator::BitNot, JavaUnaryOperator::Negate] {
                let value = JavaExpr {
                    ty: JavaType::primitive(result),
                    precedence: JavaPrecedence::Unary,
                    kind: JavaExprKind::Unary {
                        operator,
                        operand: Box::new(literal(left)),
                    },
                };
                assert_eq!(
                    admitted(value),
                    operator == JavaUnaryOperator::BitNot
                        && left != JavaPrimitive::Boolean
                        && left == result,
                    "{left:?} {operator:?} {result:?}"
                );
            }
            for right in types {
                for operator in [
                    JavaBinaryOperator::BitAnd,
                    JavaBinaryOperator::BitOr,
                    JavaBinaryOperator::BitXor,
                    JavaBinaryOperator::Add,
                    JavaBinaryOperator::ShiftLeft,
                    JavaBinaryOperator::ShiftRight,
                ] {
                    let bitwise = matches!(
                        operator,
                        JavaBinaryOperator::BitAnd
                            | JavaBinaryOperator::BitOr
                            | JavaBinaryOperator::BitXor
                    );
                    let value = JavaExpr {
                        ty: JavaType::primitive(result),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Binary {
                            operator,
                            left: Box::new(literal(left)),
                            right: Box::new(literal(right)),
                        },
                    };
                    assert_eq!(
                        admitted(value),
                        bitwise
                            && left != JavaPrimitive::Boolean
                            && left == right
                            && right == result,
                        "{left:?} {operator:?} {right:?} {result:?}"
                    );
                }
            }
        }
    }
}
