//! Java AST: operator signatures.

use super::expression_model::{JavaBinaryOperator, JavaLiteral, JavaUnaryOperator};
use super::types::{JavaKnownType, JavaPrimitive, JavaType};

pub(super) fn literal_matches_type(literal: &JavaLiteral, ty: &JavaType) -> bool {
    match literal {
        JavaLiteral::Boolean(_) => *ty == JavaType::Primitive(JavaPrimitive::Boolean),
        JavaLiteral::I32(_) => *ty == JavaType::Primitive(JavaPrimitive::Int),
        JavaLiteral::I64(_) => *ty == JavaType::Primitive(JavaPrimitive::Long),
        JavaLiteral::CharScalar(_) => *ty == JavaType::primitive(JavaPrimitive::Int),
        JavaLiteral::String(_) | JavaLiteral::Utf16Units(_) => {
            *ty == JavaType::known(JavaKnownType::String)
        }
        JavaLiteral::InternalNull(_) => matches!(
            ty,
            JavaType::Reference(_)
                | JavaType::Array { .. }
                | JavaType::Generic { .. }
                | JavaType::Wildcard { .. }
                | JavaType::TypeVariable(_)
        ),
    }
}

pub(super) fn unary_signature_matches(
    operator: JavaUnaryOperator,
    operand: &JavaType,
    result: &JavaType,
) -> bool {
    match operator {
        JavaUnaryOperator::Not => {
            *operand == JavaType::Primitive(JavaPrimitive::Boolean) && operand == result
        }
        JavaUnaryOperator::Negate => is_numeric_primitive(operand) && operand == result,
        JavaUnaryOperator::BitNot => is_integral_primitive(operand) && operand == result,
    }
}

pub(super) fn binary_signature_matches(
    operator: JavaBinaryOperator,
    left: &JavaType,
    right: &JavaType,
    result: &JavaType,
) -> bool {
    let boolean = JavaType::Primitive(JavaPrimitive::Boolean);
    match operator {
        JavaBinaryOperator::LogicalAnd | JavaBinaryOperator::LogicalOr => {
            left == &boolean && right == &boolean && result == &boolean
        }
        JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual => {
            invocation_types_match(left, right) && result == &boolean
        }
        JavaBinaryOperator::Less
        | JavaBinaryOperator::LessEqual
        | JavaBinaryOperator::Greater
        | JavaBinaryOperator::GreaterEqual => {
            is_numeric_primitive(left) && left == right && result == &boolean
        }
        JavaBinaryOperator::Add
        | JavaBinaryOperator::Subtract
        | JavaBinaryOperator::Multiply
        | JavaBinaryOperator::Divide
        | JavaBinaryOperator::Remainder => {
            is_numeric_primitive(left) && left == right && result == left
        }
        JavaBinaryOperator::BitAnd | JavaBinaryOperator::BitOr | JavaBinaryOperator::BitXor => {
            (is_integral_primitive(left) || left == &boolean) && left == right && result == left
        }
        JavaBinaryOperator::ShiftLeft | JavaBinaryOperator::ShiftRight => {
            is_integral_primitive(left)
                && *right == JavaType::Primitive(JavaPrimitive::Int)
                && result == left
        }
    }
}

fn is_numeric_primitive(ty: &JavaType) -> bool {
    matches!(
        ty,
        JavaType::Primitive(JavaPrimitive::Int | JavaPrimitive::Long | JavaPrimitive::Double)
    )
}

fn is_integral_primitive(ty: &JavaType) -> bool {
    matches!(
        ty,
        JavaType::Primitive(JavaPrimitive::Int | JavaPrimitive::Long)
    )
}

pub(super) fn invocation_types_match(left: &JavaType, right: &JavaType) -> bool {
    left == right
        || matches!(
            (left, right),
            (JavaType::Primitive(left), JavaType::Boxed(right))
                | (JavaType::Boxed(left), JavaType::Primitive(right))
                if left == right
        )
}
