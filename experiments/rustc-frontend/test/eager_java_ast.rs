//! Inspect actual primitive Boolean operator/type/precedence metadata.
use super::{EagerBooleanInput, EagerBooleanOperator};
use portable_backend_java::ast::*;

pub(super) fn check(input: &EagerBooleanInput<'_>, value: &JavaExpr) {
    assert_eq!(value.ty, JavaType::primitive(JavaPrimitive::Boolean));
    let JavaExprKind::Binary {
        operator,
        left,
        right,
    } = &value.kind
    else {
        panic!("eager binary")
    };
    let (expected, precedence) = match input.operator() {
        EagerBooleanOperator::And => (JavaBinaryOperator::BitAnd, JavaPrecedence::BitAnd),
        EagerBooleanOperator::Or => (JavaBinaryOperator::BitOr, JavaPrecedence::BitOr),
        EagerBooleanOperator::Xor => (JavaBinaryOperator::BitXor, JavaPrecedence::BitXor),
    };
    assert_eq!(*operator, expected);
    assert_eq!(value.precedence, precedence);
    assert_eq!(left.ty, value.ty);
    assert_eq!(right.ty, value.ty);
    eprintln!("EAGER_AST\tjava");
}
