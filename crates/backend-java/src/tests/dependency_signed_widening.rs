//! The dependency reader enforces its exact cast contract independently of AST checks.
use super::*;

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
fn signed_widening_reader_requires_exact_primitive_types_and_precedence() {
    let types = [
        JavaPrimitive::Byte,
        JavaPrimitive::Char,
        JavaPrimitive::Int,
        JavaPrimitive::Long,
        JavaPrimitive::Double,
        JavaPrimitive::Boolean,
        JavaPrimitive::Void,
    ]
    .into_iter()
    .map(JavaType::primitive)
    .chain([
        JavaType::Boxed(JavaPrimitive::Int),
        JavaType::Boxed(JavaPrimitive::Long),
        JavaType::known(crate::dialect::JavaKnownType::String),
    ])
    .collect::<Vec<_>>();
    for input in &types {
        for target in &types {
            for result in &types {
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
                    let value = JavaExpr {
                        ty: result.clone(),
                        precedence,
                        kind: JavaExprKind::Cast {
                            target: target.clone(),
                            value: Box::new(JavaExpr::local(
                                input.clone(),
                                JavaIdentifier::new("input").unwrap(),
                            )),
                        },
                    };
                    assert_eq!(
                        admitted(&value),
                        *input == JavaType::primitive(JavaPrimitive::Int)
                            && *target == JavaType::primitive(JavaPrimitive::Long)
                            && result == target
                            && precedence == JavaPrecedence::Unary,
                        "{input:?} -> {target:?}/{result:?} {precedence:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn signed_widening_reader_visits_the_original_operand_and_charges_each_node() {
    let wrap = |value| JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Long),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: JavaType::primitive(JavaPrimitive::Long),
            value: Box::new(value),
        },
    };
    let one = || JavaExpr::literal(JavaType::primitive(JavaPrimitive::Int), JavaLiteral::I32(1));
    let bad = JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Int),
        precedence: JavaPrecedence::Multiplicative,
        kind: JavaExprKind::Binary {
            operator: JavaBinaryOperator::Divide,
            left: Box::new(one()),
            right: Box::new(one()),
        },
    };
    assert!(!admitted(&wrap(bad)));
    let value = wrap(one());
    for remaining in [1, 2] {
        let mut budget = Budget { remaining };
        let success = Reader {
            methods: &BTreeMap::new(),
            records: &BTreeMap::new(),
            constants: &BTreeMap::new(),
            budget: &mut budget,
            calls: BTreeSet::new(),
            imported_height: 0,
            mutable_bools: BTreeSet::new(),
        }
        .expression(&value, 0)
        .is_ok();
        assert_eq!(success, remaining == 2);
    }
}
