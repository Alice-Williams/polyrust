//! All arithmetic is checked in the actual promoted target domain.
use super::super::CSafetyError as E;
use super::{CInteger, CNumber, numeric::limits};
use crate::ast::{
    CBinaryOperator as B, CScalarRepresentation as R, CScalarType, CUnaryOperator as U,
};

impl CNumber {
    pub(in crate::ownership) fn unary(self, operator: U) -> Result<Self, E> {
        let ty = operator.result_type(self.ty())?;
        if operator == U::LogicalNot {
            return boolean(!self.truth());
        }
        let value = self.convert(ty)?;
        match value {
            Self::Double(value) => Ok(Self::Double(-value)),
            Self::Integer(value) => {
                let result = match operator {
                    U::Negate => -value.value(),
                    U::BitNot => !value.value(),
                    U::LogicalNot => unreachable!(),
                };
                Ok(Self::Integer(finish(ty, result)?))
            }
        }
    }

    pub(in crate::ownership) fn binary(self, operator: B, right: Self) -> Result<Self, E> {
        let result = operator.result_type(self.ty(), right.ty())?;
        if matches!(operator, B::LogicalAnd | B::LogicalOr) {
            return boolean(if operator == B::LogicalAnd {
                self.truth() && right.truth()
            } else {
                self.truth() || right.truth()
            });
        }
        if matches!(operator, B::ShiftLeft | B::ShiftRight) {
            return shift(operator, self.integer()?.convert(result)?, right.integer()?);
        }
        let common = self.ty().usual_arithmetic_conversion(right.ty());
        let left = self.convert(common)?;
        let right = right.convert(common)?;
        match (left, right) {
            (Self::Double(left), Self::Double(right)) => floating(operator, left, right),
            (Self::Integer(left), Self::Integer(right)) => integer(operator, left, right),
            _ => unreachable!("usual conversion fixes one numeric domain"),
        }
    }
}

pub(super) fn boolean(value: bool) -> Result<CNumber, E> {
    Ok(CNumber::Integer(CInteger::checked(
        CScalarType::Int,
        i128::from(value),
    )?))
}

fn finish(ty: CScalarType, value: i128) -> Result<CInteger, E> {
    match ty.representation() {
        R::Unsigned(width) => CInteger::checked(ty, value.rem_euclid(1_i128 << width.bits())),
        R::Signed(_) => CInteger::checked(ty, value).map_err(|_| E::SignedOverflow),
        _ => unreachable!("promoted integer result"),
    }
}

fn integer(operator: B, left: CInteger, right: CInteger) -> Result<CNumber, E> {
    let ty = left.ty();
    let (a, b) = (left.value(), right.value());
    let comparison = match operator {
        B::Equal => Some(a == b),
        B::NotEqual => Some(a != b),
        B::Less => Some(a < b),
        B::LessEqual => Some(a <= b),
        B::Greater => Some(a > b),
        B::GreaterEqual => Some(a >= b),
        _ => None,
    };
    if let Some(value) = comparison {
        return boolean(value);
    }
    if matches!(operator, B::Divide | B::Remainder) {
        if b == 0 {
            return Err(E::DivisionByZero);
        }
        if matches!(ty.representation(), R::Signed(_)) && a == limits(ty)?.0 && b == -1 {
            return Err(E::SignedOverflow);
        }
    }
    // A U64 product can exceed i128. Compute unsigned arithmetic in u128,
    // then mask the actual target width; no host overflow changes C semantics.
    if let R::Unsigned(width) = ty.representation()
        && matches!(operator, B::Add | B::Subtract | B::Multiply)
    {
        let (a, b) = (a as u128, b as u128);
        let value = match operator {
            B::Add => a.wrapping_add(b),
            B::Subtract => a.wrapping_sub(b),
            B::Multiply => a.wrapping_mul(b),
            _ => unreachable!(),
        };
        let value = value & ((1_u128 << width.bits()) - 1);
        return Ok(CNumber::Integer(CInteger::checked(ty, value as i128)?));
    }
    let value = match operator {
        B::Add => a + b,
        B::Subtract => a - b,
        B::Multiply => a * b,
        B::Divide => a / b,
        B::Remainder => a % b,
        B::BitAnd => a & b,
        B::BitOr => a | b,
        B::BitXor => a ^ b,
        _ => unreachable!("non-arithmetic operators handled separately"),
    };
    Ok(CNumber::Integer(finish(ty, value)?))
}

fn shift(operator: B, left: CInteger, right: CInteger) -> Result<CNumber, E> {
    let width = match left.ty().representation() {
        R::Signed(width) | R::Unsigned(width) => width.bits(),
        _ => unreachable!("integer promotion"),
    };
    let count = right.value();
    if count < 0 || count >= i128::from(width) {
        return Err(E::InvalidShift);
    }
    let count = count as u32;
    let value = left.value();
    if operator == B::ShiftRight {
        return Ok(CNumber::Integer(CInteger::checked(
            left.ty(),
            value >> count,
        )?));
    }
    if matches!(left.ty().representation(), R::Signed(_)) && value < 0 {
        return Err(E::InvalidShift);
    }
    let shifted = (value as u128) << count;
    let result = match left.ty().representation() {
        R::Unsigned(_) => (shifted & ((1_u128 << width) - 1)) as i128,
        R::Signed(_) => {
            if shifted > limits(left.ty())?.1 as u128 {
                return Err(E::InvalidShift);
            }
            shifted as i128
        }
        _ => unreachable!(),
    };
    Ok(CNumber::Integer(CInteger::checked(left.ty(), result)?))
}

fn floating(operator: B, left: f64, right: f64) -> Result<CNumber, E> {
    Ok(CNumber::Double(match operator {
        B::Add => left + right,
        B::Subtract => left - right,
        B::Multiply => left * right,
        B::Divide => left / right,
        B::Equal => return boolean(left == right),
        B::NotEqual => return boolean(left != right),
        B::Less => return boolean(left < right),
        B::LessEqual => return boolean(left <= right),
        B::Greater => return boolean(left > right),
        B::GreaterEqual => return boolean(left >= right),
        _ => unreachable!("operator typing excludes non-floating operators"),
    }))
}
