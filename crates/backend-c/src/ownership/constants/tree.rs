//! Evaluate selected numeric children; callers first validate all syntax.
use super::super::{CSafetyError as E, layout::Layouts, ranges::CScalarRange};
use super::{CInteger, CNumber, integer_operations::boolean, known_values::known};
use crate::ast::{
    CBinaryOperator, CConversion, CLiteral, CObjectTypeKind, CScalarType, CSignedLiteral as S,
    CUnsignedLiteral as U, CValue, CValueKind as V,
};

pub(in crate::ownership) fn evaluate(
    layouts: &mut Layouts<'_>,
    value: &CValue,
) -> Result<CNumber, E> {
    match value.kind() {
        V::Literal(literal) => literal_value(literal).map(CNumber::Integer),
        V::KnownConstant(value) => known(*value).map(CNumber::Integer),
        V::Enumerator(value) => {
            CInteger::checked(CScalarType::Int, i128::from(value.value())).map(CNumber::Integer)
        }
        V::SizeOf(ty) => {
            CInteger::checked(CScalarType::Size, i128::from(layouts.object(ty)?.size()))
                .map(CNumber::Integer)
        }
        V::AlignOf(ty) => CInteger::checked(
            CScalarType::Size,
            i128::from(layouts.object(ty)?.alignment()),
        )
        .map(CNumber::Integer),
        V::Unary { operator, operand } => CScalarRange::exact(evaluate(layouts, operand)?)
            .unary(*operator)?
            .range()
            .exact_value()
            .ok_or(E::ExpectedNumericConstant),
        V::Binary {
            operator,
            left,
            right,
        } => {
            let left = evaluate(layouts, left)?;
            match operator {
                CBinaryOperator::LogicalAnd if !left.truth() => boolean(false),
                CBinaryOperator::LogicalOr if left.truth() => boolean(true),
                _ => CScalarRange::exact(left)
                    .binary(*operator, CScalarRange::exact(evaluate(layouts, right)?))?
                    .range()
                    .exact_value()
                    .ok_or(E::ExpectedNumericConstant),
            }
        }
        V::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            let selected = if evaluate(layouts, condition)?.truth() {
                then_value
            } else {
                else_value
            };
            let CObjectTypeKind::Scalar(ty) = value.ty().kind() else {
                return Err(E::ExpectedNumericConstant);
            };
            CScalarRange::exact(evaluate(layouts, selected)?)
                .convert(*ty)?
                .range()
                .exact_value()
                .ok_or(E::ExpectedNumericConstant)
        }
        V::Convert {
            conversion: CConversion::Numeric(ty),
            operand,
        } => CScalarRange::exact(evaluate(layouts, operand)?)
            .convert(*ty)?
            .range()
            .exact_value()
            .ok_or(E::ExpectedNumericConstant),
        V::Read(_)
        | V::AddressOf(_)
        | V::FunctionAddress(_)
        | V::Call(_)
        | V::PointerTest(_)
        | V::Convert { .. } => Err(E::ExpectedNumericConstant),
    }
}

fn literal_value(literal: &CLiteral) -> Result<CInteger, E> {
    let value = match literal {
        CLiteral::Bool(value) => i128::from(*value),
        CLiteral::CharByte(value) => i128::from(*value),
        CLiteral::Signed(value) => match value {
            S::PlainChar(value) | S::I8(value) => i128::from(*value),
            S::Int(value) | S::I32(value) => i128::from(*value),
            S::I16(value) => i128::from(*value),
            S::I64(value) => i128::from(*value),
        },
        CLiteral::Unsigned(value) => match value {
            U::U8(value) => i128::from(*value),
            U::U16(value) => i128::from(*value),
            U::U32(value) => i128::from(*value),
            U::U64(value) | U::Size(value) => i128::from(*value),
        },
        CLiteral::NullPointer(_) => return Err(E::ExpectedNumericConstant),
    };
    let ty = literal.ty();
    let CObjectTypeKind::Scalar(ty) = ty.kind() else {
        unreachable!("numeric literal")
    };
    CInteger::checked(*ty, value)
}
