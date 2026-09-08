//! Exact constant leaves, not the arithmetic/range evaluator owned by 02D.

use super::super::{
    CBinaryOperator as B, CConversion, CLiteral, CScalarType, CSignedLiteral as S, CUnaryOperator,
    CUnsignedLiteral as U, CValue, CValueKind as V,
};

pub(super) fn integer(value: &CValue) -> Option<i128> {
    Some(match value.kind() {
        V::Enumerator(value) => i128::from(value.value()),
        V::Literal(CLiteral::Bool(value)) => i128::from(*value),
        V::Literal(CLiteral::CharByte(value)) => i128::from(*value),
        V::Literal(CLiteral::Signed(value)) => match value {
            S::PlainChar(value) | S::I8(value) => i128::from(*value),
            S::I16(value) => i128::from(*value),
            S::Int(value) | S::I32(value) => i128::from(*value),
            S::I64(value) => i128::from(*value),
        },
        V::Literal(CLiteral::Unsigned(value)) => match value {
            U::U8(value) => i128::from(*value),
            U::U16(value) => i128::from(*value),
            U::U32(value) => i128::from(*value),
            U::U64(value) | U::Size(value) => i128::from(*value),
        },
        _ => return None,
    })
}

pub(super) fn truth(value: &CValue) -> Option<bool> {
    if let Some(value) = integer(value) {
        return Some(value != 0);
    }
    match value.kind() {
        V::Convert {
            conversion: CConversion::Numeric(CScalarType::Bool),
            operand,
        } => truth(operand),
        V::Unary {
            operator: CUnaryOperator::LogicalNot,
            operand,
        } => truth(operand).map(|v| !v),
        V::Binary {
            operator: B::LogicalAnd,
            left,
            right,
        } => match truth(left) {
            Some(false) => Some(false),
            Some(true) => truth(right),
            None => None,
        },
        V::Binary {
            operator: B::LogicalOr,
            left,
            right,
        } => match truth(left) {
            Some(true) => Some(true),
            Some(false) => truth(right),
            None => None,
        },
        V::Conditional {
            condition,
            then_value,
            else_value,
        } => match truth(condition) {
            Some(true) => truth(then_value),
            Some(false) => truth(else_value),
            None => None,
        },
        // Do not assume that numeric narrowing preserves nonzero, or evaluate
        // unchecked arithmetic. Those require the separate 02D proof stage.
        _ => None,
    }
}
