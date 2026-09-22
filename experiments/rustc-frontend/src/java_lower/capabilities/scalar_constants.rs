//! Fold checked constants into typed primitive scalar expressions.
use super::{ConstantInput, Mapping, ScalarConstants};
use crate::java_lower::{Reader, Result, Value, constants};

#[derive(Clone, Copy)]
pub(crate) struct JavaScalarConstants;

impl Mapping for JavaScalarConstants {
    type Capability = ScalarConstants;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, _reader: &mut Reader<'tcx>, input: ConstantInput<'tcx>) -> Result<Value> {
        let (plan, expression) = constants::expression(input.value());
        let value = Value::new(plan, expression)?;
        #[cfg(constant_ast_probe)]
        super::constant_ast::check(_reader, input, &value);
        Ok(value)
    }
}
