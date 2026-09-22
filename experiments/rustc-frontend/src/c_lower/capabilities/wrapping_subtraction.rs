//! Unsigned modulo difference and guarded signed reconstruction; no signed overflow.
use super::{Mapping, SubtractionInput, SubtractionWidth, WrappingSubtraction};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::*;

#[derive(Clone, Copy)]
pub(crate) struct CWrappingSubtraction;
impl Mapping for CWrappingSubtraction {
    type Capability = WrappingSubtraction;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: SubtractionInput<'tcx>,
    ) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(subtraction_ast_probe)]
        let start = reader.prelude.len();
        let left = reader.expr(input.left())?;
        let left = reader.materialize(left)?;
        #[cfg(subtraction_ast_probe)]
        let middle = reader.prelude.len();
        let right = reader.expr(input.right())?;
        let right = reader.materialize(right)?;
        #[cfg(subtraction_ast_probe)]
        let end_operands = reader.prelude.len();
        let (signed, unsigned, maximum, minus_one) = match input.width() {
            SubtractionWidth::I32 => (
                CScalarType::I32,
                CScalarType::U32,
                CUnsignedLiteral::U32(i32::MAX as u32),
                CSignedLiteral::I32(-1),
            ),
            SubtractionWidth::I64 => (
                CScalarType::I64,
                CScalarType::U64,
                CUnsignedLiteral::U64(i64::MAX as u64),
                CSignedLiteral::I64(-1),
            ),
        };
        if left.ty().kind() != &CObjectTypeKind::Scalar(signed) || right.ty() != left.ty() {
            return Err("wrapping subtraction C representation differs from checked width".into());
        }
        let left = c(reader.expressions().numeric_conversion(unsigned, left))?;
        let right = c(reader.expressions().numeric_conversion(unsigned, right))?;
        let difference = c(reader
            .expressions()
            .binary(CBinaryOperator::Subtract, left, right))?;
        let difference = reader.materialize(difference)?;
        let e = reader.expressions();
        let maximum = c(e.literal(CLiteral::Unsigned(maximum)))?;
        let comparison = c(e.binary(CBinaryOperator::LessEqual, difference.clone(), maximum))?;
        let condition = c(e.numeric_conversion(CScalarType::Bool, comparison))?;
        let positive = c(e.numeric_conversion(signed, difference.clone()))?;
        let complement = c(e.unary(CUnaryOperator::BitNot, difference))?;
        let magnitude = c(e.numeric_conversion(signed, complement))?;
        let minus_one = c(e.literal(CLiteral::Signed(minus_one)))?;
        let negative = c(e.binary(CBinaryOperator::Subtract, minus_one, magnitude))?;
        let result = c(e.conditional(condition, positive, negative))?;
        let result = if signed == CScalarType::I32 {
            c(e.numeric_conversion(signed, result))?
        } else {
            result
        };
        #[cfg(subtraction_ast_probe)]
        super::subtraction_ast::observe(reader, &input, &result, start, middle, end_operands);
        Ok(result)
    }
}
