//! Primitive floating unary minus; calls retain materialized source order.
use super::{FloatingInput, FloatingNegation, Mapping};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CObjectTypeKind, CScalarType, CUnaryOperator, CValue};
#[derive(Clone, Copy)]
pub(crate) struct CFloatingNegation;
impl Mapping for CFloatingNegation {
    type Capability = FloatingNegation;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;
    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: FloatingInput<'tcx>) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(floating_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.operand())?;
        let operand = reader.materialize(operand)?;
        if operand.ty().kind() != &CObjectTypeKind::Scalar(CScalarType::F64) {
            return Err("floating negation C representation differs from checked f64".into());
        }
        let result = c(reader.expressions().unary(CUnaryOperator::Negate, operand))?;
        #[cfg(floating_ast_probe)]
        super::floating_ast::check(reader, &input, &result, start);
        Ok(result)
    }
}
