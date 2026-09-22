//! Internal exact-width integer categories; no unsigned public source ABI.
use super::Node;
use crate::ast::{
    CBinaryOperator as B, CConversion, CObjectTypeKind, CScalarType as S, CUnaryOperator, CValue,
    CValueKind,
};

pub(super) fn visit<'a>(value: &'a CValue, add: &mut impl FnMut(Node<'a>)) -> bool {
    match value.kind() {
        CValueKind::Unary {
            operator: CUnaryOperator::BitNot,
            operand,
        } if unsigned(operand) && value.ty() == operand.ty() => {
            add(Node::Value(operand));
        }
        CValueKind::Binary {
            operator: B::Add | B::Subtract | B::Multiply,
            left,
            right,
        } if unsigned(left) && left.ty() == right.ty() && value.ty() == left.ty() => {
            add(Node::Value(left));
            add(Node::Value(right));
        }
        CValueKind::Binary {
            operator: B::Subtract,
            left,
            right,
        } if left.ty() == right.ty()
            && matches!(
                (scalar(left), scalar(value)),
                (Some(S::I32), Some(S::Int)) | (Some(S::I64), Some(S::I64))
            ) =>
        {
            // Numeric flow must independently prove representability.
            add(Node::Value(left));
            add(Node::Value(right));
        }
        CValueKind::Convert {
            conversion: CConversion::Numeric(target),
            operand,
        } if matches!(
            (scalar(operand), target),
            (Some(S::I32), S::U32)
                | (Some(S::I64), S::U64)
                | (Some(S::U32), S::I32)
                | (Some(S::U64), S::I64)
        ) =>
        {
            // Unsigned-to-signed conversions still require range proof.
            add(Node::Value(operand));
        }
        _ => return false,
    }
    true
}

fn scalar(value: &CValue) -> Option<S> {
    match value.ty().kind() {
        CObjectTypeKind::Scalar(ty) => Some(*ty),
        _ => None,
    }
}

fn unsigned(value: &CValue) -> bool {
    matches!(scalar(value), Some(S::U32 | S::U64))
}

#[cfg(test)]
#[path = "../../tests/shared_wrapping_subtraction_profile.rs"]
mod subtraction_tests;

#[cfg(test)]
#[path = "../../tests/shared_wrapping_multiplication_profile.rs"]
mod multiplication_tests;
