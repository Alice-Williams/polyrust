//! Fold checked constants into ordinary C literal nodes, never runtime storage.
use super::{ConstantInput, Mapping, ScalarConstants};
use crate::c_lower::{Reader, Result, c};
use crate::source_capabilities::LiteralValue;
use portable_backend_c::ast::{CLiteral, CSignedLiteral, CValue};

#[derive(Clone, Copy)]
pub(crate) struct CScalarConstants;

impl Mapping for CScalarConstants {
    type Capability = ScalarConstants;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: ConstantInput<'tcx>) -> Result<CValue> {
        let literal = match input.value() {
            LiteralValue::I32(value) => CLiteral::Signed(CSignedLiteral::I32(value)),
            LiteralValue::I64(value) => CLiteral::Signed(CSignedLiteral::I64(value)),
            LiteralValue::Bool(value) => CLiteral::Bool(value),
        };
        let value = c(reader.expressions().literal(literal))?;
        #[cfg(constant_ast_probe)]
        super::constant_ast::check(reader, input, &value);
        Ok(value)
    }
}
