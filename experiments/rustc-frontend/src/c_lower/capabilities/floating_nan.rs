//! One canonical receiver evaluation, two reads of its materialized value.
use super::{FloatingNaN, Mapping, NaNInput};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CBinaryOperator, CObjectTypeKind, CScalarType, CValue};

#[derive(Clone, Copy)]
pub(crate) struct CFloatingNaN;
impl Mapping for CFloatingNaN {
    type Capability = FloatingNaN;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: NaNInput<'tcx>) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(nan_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.receiver())?;
        let operand = reader.materialize(operand)?;
        if operand.ty().kind() != &CObjectTypeKind::Scalar(CScalarType::F64) {
            return Err("NaN classification requires an exact F64 receiver".into());
        }
        let result =
            c(reader
                .expressions()
                .binary(CBinaryOperator::NotEqual, operand.clone(), operand))?;
        let result = c(reader
            .expressions()
            .numeric_conversion(CScalarType::Bool, result))?;
        #[cfg(nan_ast_probe)]
        super::nan_ast::observe(reader, &input, &result, start);
        Ok(result)
    }
}
