//! Closed scalar inventory values, distinct from finite literal syntax.
use super::{
    CKnownConstant, CLiteral, CObjectType, CScalarType, CSignedLiteral, CUnaryOperator,
    CUnsignedLiteral, CValue, CValueKind,
};
use portable_binary64::{Binary64Sign, FiniteBinary64};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CScalarConstantValue {
    Bool(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    F64(FiniteBinary64),
    Infinity(Binary64Sign),
}

impl CScalarConstantValue {
    pub fn ty(self) -> CObjectType {
        CObjectType::scalar(match self {
            Self::Bool(_) => CScalarType::Bool,
            Self::I32(_) => CScalarType::I32,
            Self::I64(_) => CScalarType::I64,
            Self::U32(_) => CScalarType::U32,
            Self::F64(_) | Self::Infinity(_) => CScalarType::F64,
        })
    }

    /// Non-finite constants cannot be smuggled into finite literal nodes.
    pub fn literal(self) -> Option<CLiteral> {
        Some(match self {
            Self::Bool(value) => CLiteral::Bool(value),
            Self::I32(value) => CLiteral::Signed(CSignedLiteral::I32(value)),
            Self::I64(value) => CLiteral::Signed(CSignedLiteral::I64(value)),
            Self::U32(value) => CLiteral::Unsigned(CUnsignedLiteral::U32(value)),
            Self::F64(value) => CLiteral::F64(value),
            Self::Infinity(_) => return None,
        })
    }

    /// Inventory membership is reconstructed from exact certified syntax.
    /// This deliberately does not constant-fold arbitrary expressions.
    pub(crate) fn from_expression(value: &CValue) -> Option<Self> {
        let result = match value.kind() {
            CValueKind::Literal(CLiteral::Bool(value)) => Self::Bool(*value),
            CValueKind::Literal(CLiteral::Signed(CSignedLiteral::I32(value))) => Self::I32(*value),
            CValueKind::Literal(CLiteral::Signed(CSignedLiteral::I64(value))) => Self::I64(*value),
            CValueKind::Literal(CLiteral::Unsigned(CUnsignedLiteral::U32(value))) => {
                Self::U32(*value)
            }
            CValueKind::Literal(CLiteral::F64(value)) => Self::F64(*value),
            CValueKind::KnownConstant(CKnownConstant::DoubleInfinity) => {
                Self::Infinity(Binary64Sign::Positive)
            }
            CValueKind::Unary {
                operator: CUnaryOperator::Negate,
                operand,
            } if operand.ty() == &CObjectType::scalar(CScalarType::F64)
                && matches!(
                    operand.kind(),
                    CValueKind::KnownConstant(CKnownConstant::DoubleInfinity)
                ) =>
            {
                Self::Infinity(Binary64Sign::Negative)
            }
            _ => return None,
        };
        (value.ty() == &result.ty()).then_some(result)
    }
}
