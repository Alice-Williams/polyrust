//! Direct private-gate controls, independent of earlier AST certification.
use super::*;
use crate::ast::JavaPrecedence;
use crate::ast::{JavaBinaryOperator, JavaLiteral};

#[test]
fn dependency_comparisons_require_equal_widths_and_boolean_results_locally() {
    for operator in [
        JavaBinaryOperator::Equal,
        JavaBinaryOperator::NotEqual,
        JavaBinaryOperator::Less,
        JavaBinaryOperator::LessEqual,
        JavaBinaryOperator::Greater,
        JavaBinaryOperator::GreaterEqual,
    ] {
        for mixed in [false, true] {
            for reverse in [false, true] {
                for boolean_result in [false, true] {
                    let long = JavaType::primitive(JavaPrimitive::Long);
                    let wide = JavaExpr::literal(long.clone(), JavaLiteral::I64(1));
                    let other = if mixed {
                        JavaExpr::literal(
                            JavaType::primitive(JavaPrimitive::Int),
                            JavaLiteral::I32(1),
                        )
                    } else {
                        wide.clone()
                    };
                    let (left, right) = if reverse {
                        (other, wide)
                    } else {
                        (wide, other)
                    };
                    let value = JavaExpr {
                        ty: if boolean_result {
                            JavaType::primitive(JavaPrimitive::Boolean)
                        } else {
                            long
                        },
                        precedence: match operator {
                            JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual => {
                                JavaPrecedence::Equality
                            }
                            _ => JavaPrecedence::Relational,
                        },
                        kind: JavaExprKind::Binary {
                            operator,
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                    };
                    let mut budget = Budget::new();
                    let mut reader = Reader {
                        results: &Default::default(),
                        nonnull_results: BTreeSet::new(),
                        methods: &BTreeMap::new(),
                        records: &BTreeMap::new(),
                        constants: &BTreeMap::new(),
                        budget: &mut budget,
                        calls: BTreeSet::new(),
                        imported_height: 0,
                        mutable_bools: BTreeSet::new(),
                    };
                    let result = reader.expression(&value, 0);
                    assert_eq!(
                        result.is_ok(),
                        !mixed && boolean_result,
                        "{operator:?} mixed={mixed} reverse={reverse} boolean={boolean_result}"
                    );
                    if let Err(error) = result {
                        assert!(error.contains("unadmitted expression"), "{error}");
                    }
                }
            }
        }
    }
}
