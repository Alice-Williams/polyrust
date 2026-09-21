//! Ordered once-only operands become the typed standard fmod catalogue call.
use super::{FloatingRemainder, Mapping, RemainderInput};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::{
    ast::{CObjectTypeKind, CScalarType, CValue},
    dialect::CKnownCall,
};

#[derive(Clone, Copy)]
pub(crate) struct CFloatingRemainder;
impl Mapping for CFloatingRemainder {
    type Capability = FloatingRemainder;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: RemainderInput<'tcx>,
    ) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(remainder_ast_probe)]
        let start = reader.prelude.len();
        let left = reader.expr(input.left())?;
        let left = reader.materialize(left)?;
        #[cfg(remainder_ast_probe)]
        let middle = reader.prelude.len();
        let right = reader.expr(input.right())?;
        let right = reader.materialize(right)?;
        if left.ty().kind() != &CObjectTypeKind::Scalar(CScalarType::F64) || right.ty() != left.ty()
        {
            return Err("floating remainder requires exact F64 operands".into());
        }
        let expressions = reader.expressions();
        let result = c(expressions.call_value(
            expressions.known(CKnownCall::FloatRemainder),
            vec![left, right],
        ))?;
        #[cfg(remainder_ast_probe)]
        super::remainder_ast::observe(reader, &input, &result, start, middle);
        Ok(result)
    }
}
