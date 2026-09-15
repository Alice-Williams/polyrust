//! Actual mapper type, operator and precedence metadata must agree.
use super::{BitwiseInput, BitwiseOperands, BitwiseOperator};
use crate::java_lower::Reader;
use portable_backend_java::ast::*;
use rustc_middle::ty;

pub(super) fn check(reader: &Reader<'_>, input: &BitwiseInput<'_>, value: &JavaExpr) {
    let expression = match input.operands() {
        BitwiseOperands::Complement(operand) | BitwiseOperands::Binary(_, operand, _) => operand,
    };
    let scalar = if matches!(
        reader.checked.expr_ty(expression).kind(),
        ty::Int(ty::IntTy::I64)
    ) {
        JavaPrimitive::Long
    } else {
        JavaPrimitive::Int
    };
    assert_eq!(value.ty, JavaType::primitive(scalar));
    match input.operands() {
        BitwiseOperands::Complement(_) => {
            assert_eq!(value.precedence, JavaPrecedence::Unary);
            let JavaExprKind::Unary {
                operator: JavaUnaryOperator::BitNot,
                operand,
            } = &value.kind
            else {
                panic!("typed complement")
            };
            assert_eq!(operand.ty, value.ty);
        }
        BitwiseOperands::Binary(source, _, _) => {
            let JavaExprKind::Binary {
                operator,
                left,
                right,
            } = &value.kind
            else {
                panic!("typed binary")
            };
            let (expected, precedence) = match source {
                BitwiseOperator::And => (JavaBinaryOperator::BitAnd, JavaPrecedence::BitAnd),
                BitwiseOperator::Or => (JavaBinaryOperator::BitOr, JavaPrecedence::BitOr),
                BitwiseOperator::Xor => (JavaBinaryOperator::BitXor, JavaPrecedence::BitXor),
            };
            assert_eq!(*operator, expected);
            assert_eq!(value.precedence, precedence);
            assert_eq!(left.ty, value.ty);
            assert_eq!(right.ty, value.ty);
        }
    }
    eprintln!("BITWISE_AST\tjava");
}
