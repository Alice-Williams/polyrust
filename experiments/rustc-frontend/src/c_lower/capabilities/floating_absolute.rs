//! Materialize once, normalize either signed zero, then select the magnitude.
use super::{AbsoluteInput, FloatingAbsolute, Mapping};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{
    CBinaryOperator, CLiteral, CObjectTypeKind, CScalarType, CUnaryOperator, CValue,
};
#[derive(Clone, Copy)]
pub(crate) struct CFloatingAbsolute;
impl Mapping for CFloatingAbsolute {
    type Capability = FloatingAbsolute;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;
    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: AbsoluteInput<'tcx>) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(absolute_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.receiver())?;
        let operand = reader.materialize(operand)?;
        if operand.ty().kind() != &CObjectTypeKind::Scalar(CScalarType::F64) {
            return Err("Floating absolute value requires an exact F64 receiver".into());
        }
        let e = reader.expressions();
        let zero = c(e.literal(CLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(0).expect("positive zero is finite"),
        )))?;
        let negative = c(e.unary(CUnaryOperator::Negate, operand.clone()))?;
        let below = c(e.binary(CBinaryOperator::Less, operand.clone(), zero.clone()))?;
        let below = c(e.numeric_conversion(CScalarType::Bool, below))?;
        let magnitude = c(e.conditional(below, negative, operand.clone()))?;
        let equal = c(e.binary(CBinaryOperator::Equal, operand, zero.clone()))?;
        let equal = c(e.numeric_conversion(CScalarType::Bool, equal))?;
        let result = c(e.conditional(equal, zero, magnitude))?;
        #[cfg(absolute_ast_probe)]
        super::absolute_ast::observe(reader, &input, &result, start);
        Ok(result)
    }
}
