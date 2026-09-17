//! One receiver evaluation and a closed standard-library call.
use super::{FloatingTruncation, Mapping, TruncationInput};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::{
    ast::{CObjectTypeKind, CScalarType, CValue},
    dialect::CKnownCall,
};
#[derive(Clone, Copy)]
pub(crate) struct CFloatingTruncation;
impl Mapping for CFloatingTruncation {
    type Capability = FloatingTruncation;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: TruncationInput<'tcx>,
    ) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(truncation_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.receiver())?;
        let operand = reader.materialize(operand)?;
        if operand.ty().kind() != &CObjectTypeKind::Scalar(CScalarType::F64) {
            return Err("Floating truncation requires an exact F64 receiver".into());
        }
        let expressions = reader.expressions();
        let result =
            c(expressions.call_value(expressions.known(CKnownCall::FloatTruncate), vec![operand]))?;
        #[cfg(truncation_ast_probe)]
        super::truncation_ast::observe(reader, &input, &result, start);
        Ok(result)
    }
}
