//! Exact primitive constant inventory, separate from finite literal syntax.
use super::{
    JavaExpr, JavaExprKind, JavaLiteral, JavaPrecedence, JavaPrimitive, JavaType, JavaValueRef,
};
use crate::dialect::JavaKnownField;
use portable_binary64::{Binary64Sign, FiniteBinary64};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaScalarConstantValue {
    Boolean(bool),
    I32(i32),
    I64(i64),
    F64(FiniteBinary64),
    Infinity(Binary64Sign),
}

impl JavaScalarConstantValue {
    pub fn ty(self) -> JavaType {
        JavaType::primitive(match self {
            Self::Boolean(_) => JavaPrimitive::Boolean,
            Self::I32(_) => JavaPrimitive::Int,
            Self::I64(_) => JavaPrimitive::Long,
            Self::F64(_) | Self::Infinity(_) => JavaPrimitive::Double,
        })
    }

    /// Nonfinite values have no finite literal projection.
    pub fn literal(self) -> Option<JavaLiteral> {
        Some(match self {
            Self::Boolean(value) => JavaLiteral::Boolean(value),
            Self::I32(value) => JavaLiteral::I32(value),
            Self::I64(value) => JavaLiteral::I64(value),
            Self::F64(value) => JavaLiteral::F64(value),
            Self::Infinity(_) => return None,
        })
    }

    /// Exact syntax only: no folding, casts, alternate owners or field text.
    pub(crate) fn from_expression(expression: &JavaExpr) -> Option<Self> {
        let value = match &expression.kind {
            JavaExprKind::Literal(JavaLiteral::Boolean(value)) => Self::Boolean(*value),
            JavaExprKind::Literal(JavaLiteral::I32(value)) => Self::I32(*value),
            JavaExprKind::Literal(JavaLiteral::I64(value)) => Self::I64(*value),
            JavaExprKind::Literal(JavaLiteral::F64(value)) => Self::F64(*value),
            JavaExprKind::Value(JavaValueRef::KnownField(
                JavaKnownField::DoublePositiveInfinity,
            )) => Self::Infinity(Binary64Sign::Positive),
            JavaExprKind::Value(JavaValueRef::KnownField(
                JavaKnownField::DoubleNegativeInfinity,
            )) => Self::Infinity(Binary64Sign::Negative),
            _ => return None,
        };
        (expression.ty == value.ty() && expression.precedence == JavaPrecedence::Primary)
            .then_some(value)
    }
}
