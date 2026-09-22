//! Map checked scalar values to typed C literals; source semantics live upstream.
use super::{LiteralInput, LiteralValues, Mapping};
use crate::c_lower::{Reader, Result, c};
use crate::source_capabilities::LiteralValue;
use portable_backend_c::ast::{CLiteral, CSignedLiteral, CUnsignedLiteral, CValue};

#[derive(Clone, Copy)]
pub(crate) struct CLiteralValues;

impl Mapping for CLiteralValues {
    type Capability = LiteralValues;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: LiteralInput<'tcx>) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        let literal = match input.value() {
            LiteralValue::I32(value) => CLiteral::Signed(CSignedLiteral::I32(value)),
            LiteralValue::I64(value) => CLiteral::Signed(CSignedLiteral::I64(value)),
            LiteralValue::Bool(value) => CLiteral::Bool(value),
            LiteralValue::Char(value) => {
                CLiteral::Unsigned(CUnsignedLiteral::U32(u32::from(value)))
            }
            LiteralValue::F64(value) => CLiteral::F64(value),
        };
        let value = c(reader.expressions().literal(literal))?;
        #[cfg(binary64_ast_probe)]
        super::binary64_ast::check(reader, input, &value);
        Ok(value)
    }
}
