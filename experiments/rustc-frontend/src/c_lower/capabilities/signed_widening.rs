//! Lossless numeric conversion after one original operand evaluation.
use super::{Mapping, SignedWidening, WideningInput};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CObjectTypeKind, CScalarType, CValue};

#[derive(Clone, Copy)]
pub(crate) struct CSignedWidening;
impl Mapping for CSignedWidening {
    type Capability = SignedWidening;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;
    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: WideningInput<'tcx>) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(widening_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.operand())?;
        let operand = reader.materialize(operand)?;
        if operand.ty().kind() != &CObjectTypeKind::Scalar(CScalarType::I32) {
            return Err("signed widening C representation differs from checked i32".into());
        }
        let result = c(reader
            .expressions()
            .numeric_conversion(CScalarType::I64, operand))?;
        #[cfg(widening_ast_probe)]
        super::widening_ast::check(reader, &input, &result, start);
        Ok(result)
    }
}
