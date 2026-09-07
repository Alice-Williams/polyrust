//! Java lowering: expression builders.

use super::call_builders::{known_call, new_known};
use crate::ast::{
    JavaBinaryOperator, JavaExpr, JavaExprKind, JavaIdentifier, JavaKnownType, JavaLiteral,
    JavaPrecedence, JavaPrimitive, JavaType, JavaUnaryOperator,
};
use crate::dialect::{JavaKnownCallable, JavaKnownConstructor};

pub(crate) fn java_unit_value() -> JavaExpr {
    new_known(
        JavaKnownConstructor::RuntimeUnit,
        JavaType::known(JavaKnownType::RuntimeUnit),
        vec![],
    )
}

pub(crate) fn bool_literal(value: bool) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Boolean),
        JavaLiteral::Boolean(value),
    )
}

pub(crate) fn i32_literal(value: i32) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Int),
        JavaLiteral::I32(value),
    )
}

pub(crate) fn i64_literal(value: i64) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Long),
        JavaLiteral::I64(value),
    )
}

pub(crate) fn f64_literal(value: u64) -> JavaExpr {
    known_call(
        JavaKnownCallable::DoubleFromLongBits,
        vec![i64_literal(value as i64)],
    )
}

pub(crate) fn scalar_literal(value: char) -> JavaExpr {
    new_known(
        JavaKnownConstructor::RuntimeScalar,
        JavaType::known(JavaKnownType::RuntimeScalar),
        vec![JavaExpr::literal(
            JavaType::primitive(JavaPrimitive::Int),
            JavaLiteral::CharScalar(u32::from(value)),
        )],
    )
}

pub(crate) fn string_literal(value: &str) -> JavaExpr {
    JavaExpr::literal(
        JavaType::known(JavaKnownType::String),
        JavaLiteral::String(value.to_owned()),
    )
}

pub(crate) fn unary(operator: JavaUnaryOperator, operand: JavaExpr, ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator,
            operand: Box::new(operand),
        },
    }
}

pub(crate) fn binary(
    operator: JavaBinaryOperator,
    left: JavaExpr,
    right: JavaExpr,
    ty: JavaType,
) -> JavaExpr {
    let precedence = match operator {
        JavaBinaryOperator::LogicalOr => JavaPrecedence::LogicalOr,
        JavaBinaryOperator::LogicalAnd => JavaPrecedence::LogicalAnd,
        JavaBinaryOperator::BitOr => JavaPrecedence::BitOr,
        JavaBinaryOperator::BitXor => JavaPrecedence::BitXor,
        JavaBinaryOperator::BitAnd => JavaPrecedence::BitAnd,
        JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual => JavaPrecedence::Equality,
        JavaBinaryOperator::Less
        | JavaBinaryOperator::LessEqual
        | JavaBinaryOperator::Greater
        | JavaBinaryOperator::GreaterEqual => JavaPrecedence::Relational,
        JavaBinaryOperator::ShiftLeft | JavaBinaryOperator::ShiftRight => JavaPrecedence::Shift,
        JavaBinaryOperator::Add | JavaBinaryOperator::Subtract => JavaPrecedence::Additive,
        JavaBinaryOperator::Multiply
        | JavaBinaryOperator::Divide
        | JavaBinaryOperator::Remainder => JavaPrecedence::Multiplicative,
    };
    JavaExpr {
        ty,
        precedence,
        kind: JavaExprKind::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        },
    }
}

pub(crate) fn conditional(
    condition: JavaExpr,
    when_true: JavaExpr,
    when_false: JavaExpr,
    ty: JavaType,
) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
        },
    }
}

pub(super) fn instance_of(
    value: JavaExpr,
    target: JavaType,
    binding: Option<JavaIdentifier>,
) -> JavaExpr {
    JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Boolean),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::InstanceOf {
            value: Box::new(value),
            target,
            binding,
        },
    }
}
