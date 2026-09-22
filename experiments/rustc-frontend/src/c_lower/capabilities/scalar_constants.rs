//! Fold checked constants into typed scalar syntax, never runtime storage.
use super::{ConstantInput, Mapping, ScalarConstants};
use crate::c_lower::{Reader, Result, constants};
use portable_backend_c::ast::CValue;

#[derive(Clone, Copy)]
pub(crate) struct CScalarConstants;

impl Mapping for CScalarConstants {
    type Capability = ScalarConstants;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: ConstantInput<'tcx>) -> Result<CValue> {
        let value = constants::expression(&reader.registry, input.value())?;
        #[cfg(constant_ast_probe)]
        super::constant_ast::check(reader, input, &value);
        Ok(value)
    }
}
