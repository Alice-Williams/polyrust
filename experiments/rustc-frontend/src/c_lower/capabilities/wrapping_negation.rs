//! Guarded signed negation expressed entirely with existing certified C nodes.
use super::{Mapping, WrappingInput, WrappingNegation, WrappingWidth};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::*;

#[derive(Clone, Copy)]
pub(crate) struct CWrappingNegation;
impl Mapping for CWrappingNegation {
    type Capability = WrappingNegation;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: WrappingInput<'tcx>) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(wrapping_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.receiver())?;
        let operand = reader.materialize(operand)?;
        let (width, minimum) = match input.width() {
            WrappingWidth::I32 => (CScalarType::I32, CSignedLiteral::I32(i32::MIN)),
            WrappingWidth::I64 => (CScalarType::I64, CSignedLiteral::I64(i64::MIN)),
        };
        if operand.ty().kind() != &CObjectTypeKind::Scalar(width) {
            return Err("wrapping negation C representation differs from checked width".into());
        }
        let minimum = c(reader.expressions().literal(CLiteral::Signed(minimum)))?;
        let comparison = c(reader.expressions().binary(
            CBinaryOperator::Equal,
            operand.clone(),
            minimum.clone(),
        ))?;
        let condition = c(reader
            .expressions()
            .numeric_conversion(CScalarType::Bool, comparison))?;
        let negative = c(reader.expressions().unary(CUnaryOperator::Negate, operand))?;
        let value = c(reader
            .expressions()
            .conditional(condition, minimum, negative))?;
        let result = if width == CScalarType::I32 {
            c(reader
                .expressions()
                .numeric_conversion(CScalarType::I32, value))?
        } else {
            value
        };
        #[cfg(wrapping_ast_probe)]
        super::wrapping_ast::check(reader, &input, &result, start);
        Ok(result)
    }
}
